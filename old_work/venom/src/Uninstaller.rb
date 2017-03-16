require_relative "./Database.rb"
require_relative "./Config.rb"
require_relative "./DB_Utils.rb"
require_relative "./Relinker.rb"

class Uninstaller
    def initialize( package_name )
        @db = Database.instance.get_db( )
        @pkg_name = package_name
        @unbound_packages = []
    end

    def unbind( )
        @db.execute( "SELECT owner,commit_action,shared,rowid FROM bindings WHERE owner LIKE '%" + @pkg_name + "%' AND commit_action != 'DELETE';" ) do |row|
            owner = row[ 0 ]
            action = row[ 1 ]
            shared = row[ 2 ]
            rowid = row[ 3 ]
            if( shared == 0 )
                if( action == "CREATE" )
                    # This is an uncommitted change applied only to this package. We can just delete it
                    # Delete the offending records
                    @db.execute( "DELETE FROM bindings WHERE rowid = ?;", rowid )
                else
                    # Otherwise, we need to mark it for deletion
                    @db.execute( "UPDATE bindings SET commit_action = 'DELETE' WHERE rowid = ?;", rowid )
                end
            else
                # If this is a shared dir, it gets more complex
                # We essentially have three scenarios
                #   1. There are >2 packages that share this dir, and we can just, and it is sufficient to remove this
                #   package from the list of owners
                #   2. There are exactly 2 owners, in which case removing this package leaves a directory shared by only one package
                #   In this case, we should unlink the other package that shares a dir with this package, delete the actual dir, and
                #   allow the other package to relink correctly
                #   3. This is a shared dir with only one owner, which would come about as a result of two
                #   In this case, we just delete the record and call it a day
                #
                owners = owner.split( "," )
                num_owners = owners.length
                if( num_owners > 2 )
                    owners -= [ @pkg_name ]
                    num_owners -= 1
                    new_owners = ""
                    for i in 0..( num_owners - 1 )
                        new_owners.concat( owners[ i ] )
                        if( i != ( num_owners - 1 ) )
                            new_owners.concat( "," )
                        end
                    end
                    @db.execute( "UPDATE bindings SET owner = ? WHERE rowid = ?;", [ new_owners, rowid ] )
                elsif( num_owners == 2 )
                    owners -= [ @pkg_name ]
                    DB_Utils.instance.soft_unlink( owners[ 0 ] )
                    @db.execute( "UPDATE bindings SET commit_action = 'DELETE' WHERE rowid = ?;", [ rowid ] )
                    @unbound_packages += [ owners[ 0 ] ];
                elsif( num_owners == 1 )
                    @db.execute( "UPDATE bindings SET commit_action = 'DELETE' WHERE rowid = ?;", [ rowid ] )
                end
            end
        end


        # Rebind any of the packages that we've unbound during installation.
        @unbound_packages.each{ |unbound_name|
            # Resolve the unbound package name to its files
            @db.execute( "SELECT name,current_version FROM packages WHERE name = ?;", unbound_name ) do |row|
                Relinker.new( row[ 0 ], row[ 1 ] ).integrate( )
            end
        }
    end
end
