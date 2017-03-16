
require "singleton"
class DB_Utils
    include Singleton
    
    def initialize( )
        @db = Database.instance.get_db( )
    end


    # In the event of installing a new, sometimes we need to unbind to create a shared
    # dir. As we aren't really removing this package from the system, we really only
    # want it to regenerate its list of symlinks on rebind. Therefore, it will suffice to
    # remove all of its symlinks, but not remove it from the owners of a shared dir
    def soft_unlink( pkg_name )
        @db.execute( "SELECT rowid,commit_action FROM bindings WHERE owner = ? AND shared = 0;", pkg_name ) do |row|
            # The system should NEVER commit changes after soft unlinking a package,
            # so our major goal should be to leave the database in the best place to 
            # resync with the filesystem after the install is completely done
            # As committing will run thorugh all DELETE operations first, and is
            # safe whether or not the symlink actually exists or not, we can kind of let them
            # pile up. 
            # What does this mean?
            # Well, essentially, once all DELETE operations are done, the CREATE ops
            # plus the existing links and dirs MUST represent the final state of the 
            # file system. So if here, we remove all records to be created, and mark for 
            # deletion any unmarked records, we should be okay.
            #
            if( row[ 1 ] == "CREATE" )
                @db.execute( "DELETE FROM bindings WHERE rowid = ?;", row[ 0 ] )
            elsif( row[ 1 ] == "" )
                @db.execute( "UPDATE bindings SET commit_action = 'DELETE' WHERE rowid = ?;", row[ 0 ] )
            end
        end
    end 
end
