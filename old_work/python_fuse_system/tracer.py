# Test the speed of tracing a largish filesystem
from SubFS import SubFS
import os
import sys

def traceFS(rootdir):
    sfs = SubFS( rootdir )
    sub = len( rootdir ) + 1
    for root, subFolders, files in os.walk(rootdir):
        target = root[sub:] + "/"
        print( "Traced %s" % (rootdir) )
        if(target == ""):
            continue

        sfs.trace( target, True )
        for f in files:
            sfs.trace( target + "/" + f, True)
    return sfs

# query = sys.argv[2]
# print( "Begin query" )
# print("SubFS %s has %s?: %r" % (rootdir, query, sfs.trace(query, False) ) )
