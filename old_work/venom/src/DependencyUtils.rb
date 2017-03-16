require_relative "./Database.rb"
require_relative "./TransactionManager.rb"
require_relative "./PkgCrawler.rb"

# Given a list of packages, extract their dependencies and verify that
# there are no conflicts in version numbers. Additionally, we check all installed
# packages to ensure that this range falls within the ranges required by the
# system
def verify_deps( packages )
    deps = []

    # Get a list of all unique deps required by these packages
    # Each dep is an object with three attributes:
    # name, minversion, maxversion. A 0 in minversion indicates
    # that there is no minimum version, and a 0 in maxversion indicates
    # that there is no maxmimum version. Similarly, if the field does not
    # exist, it will be assumed to be zero
    # Version numbers are represented by
    # a dot separated string of ints, e.g. 1.2.3.4, where the leftmost integer
    # is the major version
    # [ "name":"vim", "minversion":"0","maxversion":"4.0"]
    packages.each{ |p|
        deps |= p.depends
    }

    # Now we need to make sure that, if we have duplicate package entries
    # with different version requirements, there exists a version that satisfies
    # the requirement. We also need to check if this package is already installed,
    # and what packages require it, and what version of it they need
    # It is entirely possible that this is unreasolvable, so we must inform the user
    # of this and cancel the transaction

    # Start with the first element in the array and search the rest of the
    # array for other copies of it. If we find no other copies, we're good, and we can
    # just add it to the array of checked deps. If we find more than one copy, we put
    # the first copy in the checked deps array and check every subsequent one against this
    # to ensure that it is a valid state
    checked = []
    while( deps.length > 0 )
        target = deps[ 0 ][ "name" ]

        # Push this to the back of the checked array
        checked.push( deps[ 0 ] )

        # Shift this element out of the deps so that we do not check it against
        # itself
        deps.shift( )

        # This will be an array without any copies of the target,
        # as we will remove them all in the next step. In this way, we
        # only end up with one copy of each package in the checked array
        deps_processed = []

        # Look for other copies of this in the deps array
        deps.each{ |d|
            if( d[ "name" ] == target )
                # We need to do a few things here:
                #   1. Check that it is compatible with the version requirements
                #      already in the checked array
                #   2. Update those if they need to become more restrictive
                #   3. Remove d from the deps array
                # This is kludgy
                new_min = (d[ "minversion" ] == nil ? "0" : d[ "minversion" ] )
                new_max = (d[ "maxversion" ] == nil ? nil : d[ "maxversion" ] )
                checked_min = (checked[ -1 ][ "minversion" ] == nil ? "0" : checked[ -1 ][ "minversion" ] )
                checked_max = (checked[ -1 ][ "maxversion" ] == nil ? nil : checked[ -1 ][ "maxversion" ] )
                if( hasOverlap?( [ new_min, new_max ], [ checked_min, checked_max ] ) )
                    vrange = getIdealRange( [ new_min, new_max, checked_min, checked_max ] )
                    checked[ -1 ][ "minversion" ] = vrange[ 0 ]
                    checked[ -1 ][ "maxversion" ] = vrange[ 1 ]
                else
                    # Not resolvable
                    puts "ERR: Unresolveable dependency for dependency %s" % [ d[ "name" ] ]
                    return nil
                end
            else
                deps_processed.push( d )
            end
        }

        # Perform our substitution which entirely removes this target from the deps
        deps = deps_processed
    end

    # At this point, the checked array now contains the largest permissable range
    # of dependency versions for each package. What we need to do now is verify that
    # this range is attainable within the constraints of the packages already installed.
    # If this not possible, we cannot proceed with this transaction, and we must abort
    checked.each{ |pkg|
        # Get all of the rows
        # i.e. get all records in the DB that indicate this package as a dependency
        vRange = { "min"=>pkg[ "minversion" ], "max"=>pkg[ "maxversion" ] }
        Database.instance.get_db( ).execute( "SELECT * FROM dependencies WHERE name = ?;", pkg[ "name" ] ) do |row|
            if( hasOverlap?( [ vRange[ "min" ], vRange[ "max" ] ], [ row[ 2 ], row[ 3 ] ] ) )
                iRange = getIdealRange(  [ vRange[ "min" ], vRange[ "max" ], row[ 2 ], row[ 3 ] ] )
                vRange[ "min" ] = iRange[ 0 ]
                vRange[ "max" ] = iRange[ 1 ]
            else
                puts "Unreasolveable dependency %s, no suitable version found!" % [ pkg[ "name" ] ]
                puts "This usually means that the packages you have installed on your system require a specific version of one of the dependencies of this transaction that is different from what's required here."
                return nil
            end
        end

    }
    # If we get here, we can assume that our packages are all okay and that
    # everything is kosher in terms of version. Short story, we have a
    # resolveable state
    return checked
end

# Given a pair of tuples, each of which has the form of (minversion,maxversion),
# determine if there is overlap between the two
def hasOverlap?( vArr1, vArr2 )
    nminVcmin = compareVersion( vArr1[ 0 ].to_s( ), vArr2[ 0 ].to_s( ) )
    nminVcmax = compareVersion( vArr1[ 0 ].to_s( ), vArr2[ 1 ].to_s( ) )
    nmaxVcmin = compareVersion( vArr1[ 1 ].to_s( ), vArr2[ 0 ].to_s( ) )
    # Scenarios that are okay
    # new_min >= checked_min && new_min <= checked_max
    #    nmin      nmax            nmin     nmax
    #  |--|-----|---|     and    |--|--------|---|
    # cmin     cmax             cmin            cmax
    case1 = ( nminVcmin == 1 || nminVcmin == 0 ) &&
            ( nminVcmax == -1 || nminVcmax == 0)

    # new_min <= checked_min && new_max >= checked_min
    # nmin       nmax           nmin                nmax
    #  |---|------|----|    and  |---|--------|------|
    #     cmin       cmax           cmin     cmax
    case2 = ( nminVcmin == -1 || nminVcmin == 0 ) &&
            ( nmaxVcmin == 1  || nmaxVcmin == 0 )

    # In either scenario, we want to take the two innermost constraints
    # and set them as our target range
    return case1 || case2
end


# Takes an array of 4 version numbers, and returns the two (in an array)
# that fall in between. nil is understood to be an absolute maximum
def getIdealRange( vArr )
    sorted = vArr.sort { |x,y| compareVersion( x, y ) }
    return sorted[ 1, 2 ]
end

# Parse out a version string and return if one is larger than the other
# Will return 1 if the first is newer, -1 if the second is newer, and 0 if
# they are the same
def compareVersion( v1, v2 )
    if( v1.nil? )
        return 1
    end
    if( v2.nil? )
        return -1
    end
    t1 = v1.split( "." )
    t2 = v2.split( "." )
    for i in 0..(t1.length)
        # In the case of 1.2 vs 1.2.1, 1.2.1 is the more recent
        # version
        if( i > t2.length )
            return 1
        end

        if( t1[ i ].to_i( ) > t2[ i ].to_i( ) )
            return 1
        elsif( t1[ i ].to_i( ) < t2[ i ].to_i( ) )
            return -1
        end

        if( i == ( t1.length - 1 ) && t1.length < t2.length )
            return -1
        end
    end

    return 0
end

