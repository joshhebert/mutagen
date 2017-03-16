require_relative "./Database.rb"
require_relative "./Config.rb"
require_relative "./PkgCrawler.rb"
require_relative "./DB_Utils.rb"
require 'fileutils'

class Installer
    attr_reader :deps

    require_relative "./Relinker.rb"
    # Does 2 things:
    #   1. Extracts the package into $WORK_DIR/$NAME-$VER
    #   2. Reads data into JSON structure for the rest of the installer
    #      to use
    # If relink is set to true, we assume the package has already been
    # installed, and we can use PkgCrawler.resolve to get its data
    def initialize( package_archive )
        package = PkgCrawler.instance.map(
            Config.instance.work_dir + "/" +
            PkgCrawler.instance.unpack( package_archive )
        )

        @db = Database.instance.get_db( )
        @unbound = []
        @pkg_files = package.package_files
        @pkg_name = package.name
        @pkg_ver = package.version
        @deps = package.depends
        @depth = 0
    end

    def integrate( )

        # Mark this in the installed packages table. If this package is already
        # in the table, update the installed version and make sure this is in the
        # list of available versions
        cnt = 0
        rows = []
        @db.execute( "SELECT rowid,available_versions FROM packages WHERE name = ?;", @pkg_name ) do |row|
            rows += row
        end
        if( rows.length == 2 )
            available_vers = rows[ 1 ].split( "," )
            if( !( available_vers.include?( @pkg_ver ) ) )
                new_versions = ""
                for i in 0..( available_vers.length - 1 )
                    new_versions.concat( available_vers[ i ] )
                    new_versions.concat( "," )
                end
                new_versions.concat( @pkg_ver )
                @db.execute( "UPDATE packages SET available_versions = ? WHERE rowid = ?;", [ new_versions, rows[ 0 ] ] )
            end
            @db.execute( "UPDATE packages SET current_version = ? WHERE rowid = ?;", [ @pkg_ver, rows[ 0 ] ] )

            # In this instance, we are doing an upgrade or a downgrade, so we should preemptively soft-unlink
            # this package in the DB
            DB_Utils.instance.soft_unlink( @pkg_name )

        elsif( rows.length > 2 )
            # This should NEVER happen
            puts "This should never happen. Email Josh"
        else
            @db.execute( "INSERT INTO packages (name,current_version,available_versions) VALUES (?,?,?);", [ @pkg_name, @pkg_ver, @pkg_ver ] )
        end

        merge_fs( @pkg_files )

        @unbound.each{ |unbound_name|
            # Resolve the unbound package name to its files
            @db.execute( "SELECT name,current_version FROM packages WHERE name = ?;", unbound_name ) do |row|
                # Relinker is a subclass of installer that, rather than applying a txz
                # just performs the relinking steps for a package already installed into
                # the install dir
                Relinker.new( row[ 0 ], row[ 1 ] ).integrate( )
            end
        }

        # We now need to go into our dependencies table, remove any entries that list this as
        # the owner. This is necessary to do when this installation is actually an upgrade.
        # In the future, we could check if dependency requirements have changed, and only perform
        # this operation if necessary, but for now, this is fine
        @db.execute( "DELETE FROM dependencies WHERE owner = ?;", @pkg_name )
        # We now insert this packages dependencies
        @deps.each{ |d|
            @db.execute( "INSERT INTO dependencies (name,owner,minversion,maxversion) VALUES (?,?,?,?);",
                        [ d[ "name" ], @pkg_name, d[ "minversion" ], d[ "maxversion" ] ] )
        }


        # Finally, copy all of the files in the work dir into our install dir
        target_dir = Config.instance.install_dir + "/" + @pkg_name + "/" + @pkg_ver
        if( !File.directory?( target_dir ) )
            FileUtils.mkdir_p( target_dir )
        end
        src_dir = Config.instance.work_dir + "/" + @pkg_name + "-" + @pkg_ver + "/."
        FileUtils.cp_r( src_dir, target_dir )
    end

    def merge_fs( file_tree )
        # Start a root and select any conflicts
        file_tree.children.each{ |name, child|
            # Resolve conflicts for this child
            # Select any symlink which will cause problems linking this
            # i.e. any links that either exist or will be created that bind this dir
            # Under normal circumstances, this shouldn't return more than one row

            # Differentiate between no rows returned and a break
            pass = 0
            @db.execute( "SELECT owner,commit_action,shared FROM bindings WHERE rootfs_loc = ? AND commit_action != 'DELETE';", name ) do |row|
                prev_owner = row[ 0 ]
                commit_action = row[ 1 ]
                shared = row[ 2 ]

                if( shared == 1 )
                    owners = prev_owner.split( "," )
                    if( !( owners.include?( @pkg_name ) ) )
                        # As this node is shared, i.e. a physical dir, we can bind below here. All we need to do is add ourselved to
                        # the list of owners
                        @db.execute( "UPDATE bindings SET owner=? WHERE rootfs_loc = ?;", [ prev_owner + "," + @pkg_name, name ] )
                    end
                    # We can now trash all other rows returned.
                    # In theory, when the installer for each of them ran, it would have done this merge as well,
                    # so this query really shouldn't return more than one record

                    # And then merge
                    @depth += 1
                    merge_fs( child )
                    pass = 1
                    break
                else
                    # This is not shared, so we must unbind the owner, conver this to shared, and bind
                    if( prev_owner != @pkg_name )
                        # Manage removing bindings we don't need anymore
                        DB_Utils.instance.soft_unlink( prev_owner )


                        install_dir = "%s/%s/%s" % [ Config.instance.install_dir, @pkg_name, @pkg_ver ]

                        # Insert a shared link with a CREATE flag and the owners as the previous owner and this package
                        @db.execute( "INSERT INTO bindings (rootfs_loc,pkg_loc,owner,commit_action,shared) VALUES (?,?,?,'CREATE',1);",[ name, nil, prev_owner + "," + @pkg_name ] )

                        # As we've unbound this package, we need to make sure this is recorded, so we can rebind later
                        if( !( @unbound.include?( prev_owner ) ) )
                            puts "Unbinding %s" % prev_owner
                            @unbound += [ prev_owner ]
                        end

                        # We now bind underneath this node
                        @depth += 1
                        merge_fs( child )
                        pass = 1
                        break
                    else
                        if( @depth == 0 )
                            puts "Nothing to do!"
                        end
                        pass = 1
                        break
                    end
                end

            end
            if( pass == 0 )
                # If we got all the way here, the DB returned no records, so there are no conflicts
                # We can bind here
                install_dir = "%s/%s/%s/pkg_files" % [ Config.instance.install_dir, @pkg_name, @pkg_ver ]
                @db.execute( "INSERT INTO bindings (rootfs_loc,pkg_loc,owner,commit_action,shared) VALUES (?,?,?,'CREATE',0);",[ name, install_dir + name, @pkg_name ] )
            end
        }


    end

end
