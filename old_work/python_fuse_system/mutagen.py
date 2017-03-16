#!/usr/bin/env python

from fuse_logic import Mutagen

import sys
import os
from fuse import FUSE
from tracer import traceFS
import pytoml as toml

def main(mountpoint, meta, writeable):
    FUSE(Mutagen(meta, writeable), mountpoint, nothreads=True, foreground=True, dev=True, suid=True, nonempty=True)

if __name__ == '__main__':
    # Eventually, we will be able to add SubFS systems on the fly, but for
    # now, just grab all the sub directories in the root filesystem

    # Locate and read mutagen.toml
    with open( os.getcwd() + "/mutagen.toml" ) as conf:
        config = toml.load( conf )
        dirs = config["Subfilesystems"]["paths"]
        #writeable = config["Subfilesystems"]["writeable"]
        writeable = None


    mnt = sys.argv[1]
    subfilesytems = []

    for d in dirs:
         subfilesytems.append( traceFS( os.getcwd() + "/" + d ) )


    main(mnt, subfilesytems, writeable)
