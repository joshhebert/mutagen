# Represents the details of this specific transaction, i.e.
# - What packages are we adding/removing?
# - What is the current status?
# And so on
#

class Transaction
    def initialize
        @stage = 0
    end
    def addPackage( p )
    end

    def removePackage( p )
    end

    # Place all requested packages in the install dir, make a backup of the database, and
    # write all the expected changes to it.
    def stageChanges( )
    end

    # Apply the changes in the DB to the filesystem
    def commitChanges( )

    end
end
