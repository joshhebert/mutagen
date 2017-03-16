
require "singleton"

class Database
    include Singleton

    def initialize( )
        @db = SQLite3::Database.new( "packages.db" )
    end
    def commit( )
        @db.execute( "DELETE FROM bindings WHERE commit_action = 'DELETE';" )
        @db.execute( "UPDATE bindings SET commit_action = NULL WHERE commit_action = 'CREATE';" ) 
    end    
    def get_db( )
        return @db
    end

end
