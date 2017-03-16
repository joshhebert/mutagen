# Describes a branch of the stacked filesystem
import os
class SubFS():
    def __init__(self, name):
        self.ROOT = Node( "/" )
        self.name = os.path.abspath(name)

    def trace( self, path, create ):
        if( path == "/" ):
            return True

        p = path.split( "/" )
        p = list(filter(lambda x: x != '', p))
        return self.ROOT.trace( p, create )

    def size( self ):
        return self.ROOT.size()

    def childAt( self, path ):
        while len(path) > 0:
            n = self.ROOT.children[path[0]]
            if( n == None ):
                return False
            del path[0]
        return n


class Node( ):
    def __init__(self, name):
        self.name = name
        self.children = dict()


    def size( self ):
        sum = 0
        for c in self.children:
            sum += c.size()

        sum += len( self.children )

        return sum

    def trace( self, path, create ):
        if path[0] in self.children:
            node = self.children[path[0]]
            if( len( path ) == 1 ):
                return True
            return node.trace( path[1:], create )
        else:
            if create:
                n = Node( path[0] )
                self.children[ path[0] ] = n
                print( "Added %s to tree %s" %( path, self.name ) )
                del path[0]
                if( len(path) == 0 ):
                    return True
                else:
                    return n.trace( path, create )
            else:
                return False

