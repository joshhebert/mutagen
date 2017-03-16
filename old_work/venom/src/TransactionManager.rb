# Ensures consistency in the state of the world
# - Did we correctly finish the last transaction? If not, can we
#   recover without user intervention?
#
# This can be queried to get the current transaction, or the details of any
# past transaction
class TransactionManager
    def initialize( )
    end

    # Verify that the last transaction completed correctly, and that
    # the database accurately reflects the state of the system
    def checkConsistency( )
    end

    # Get the current transaction
    def getCurrentTransaction( )
    end


end
