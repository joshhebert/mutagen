#!/usr/bin/env python

from __future__ import with_statement

import os
import errno

from fuse import FuseOSError, Operations, fuse_get_context
from errno import ENOENT, EACCES

# Filesystem core logic
class Mutagen(Operations):
    def __init__(self, meta, writeable):
        self.metadata = meta
        self.writeable = writeable

    # Helpers
    # =======
    def _full_path(self, partial):
        if partial.startswith("/"):
            partial = partial[1:]
        path = os.path.join("/", partial)
        return path

    # Filesystem methods
    # ==================
    def access(self, path, mode):
        print( "access(self, %s, %s)" % (path, mode) )
        accessible = False
        for sfs in self.metadata:
            if( sfs.trace( path, False ) ):
                accessible = True
                break

        if not accessible:
            raise FuseOSError(errno.EACCES)

    def chmod(self, path, mode):
        print( "chmod(self, %s, %s)" % (path, mode))
        raise FuseOSError(errno.EACCES)

    def chown(self, path, uid, gid):
        print("chown(self, %s, %s, %s)" % (path, uid, gid))
        raise FuseOSError(errno.EACCES)

    def getattr(self, path, fh=None):
        print("getattr(self, %s, %s)" % (path, fh))
        full_path = None
        if( path == "/" ):
            full_path = self._full_path(path)
        else:
            p = None
            for sfs in self.metadata:

                if( sfs.trace( path, False ) ):
                    p = sfs.name + path
            full_path = p

        if(full_path == None):
            raise FuseOSError(ENOENT)

        st = os.lstat(full_path)
        return dict((key, getattr(st, key)) for key in ('st_atime', 'st_ctime',
                     'st_gid', 'st_mode', 'st_mtime', 'st_nlink', 'st_size', 'st_uid'))

    def readdir(self, path, fh):
        print( "readdir(self, %s, %s)" % (path, fh) )

        uid, gid, pid = fuse_get_context()
        encoded = lambda x: ('%s\n' % x)

        print( "user = %s" % (encoded(gid)) )

        dirents = ['.', '..']
        for sfs in self.metadata:
            if( sfs.trace( path, False ) ):
                if os.path.isdir(sfs.name  + path):
                    dirents.extend(os.listdir(sfs.name + path))

        for r in list(set(dirents)):
            yield r


    def readlink(self, path):
        print( "readlink(self, %s)" % (path) )
        p = None
        for sfs in self.metadata:
            if( sfs.trace( path, False ) ):
                p = sfs.name + path
        full_path = p

        if(full_path == None):
            raise FuseOSError(ENOENT)
        pathname = os.readlink(full_path)
        return pathname

    def mknod(self, path, mode, dev):
        print("mknod(self, %s, %s, %s)" % (path, mode, dev) )
        raise FuseOSError(errno.EACCES)

    def rmdir(self, path):
        raise FuseOSError(errno.EACCES)

    def mkdir(self, path, mode):
        print("mkdir(self, %s, %s)" % (path, mode))
        raise FuseOSError(errno.EACCES)

    def statfs(self, path):
        print( "statfs(self, %s)" %(path) )
        full_path = self._full_path(path)
        stv = os.statvfs(full_path)
        return dict((key, getattr(stv, key)) for key in ('f_bavail', 'f_bfree',
            'f_blocks', 'f_bsize', 'f_favail', 'f_ffree', 'f_files', 'f_flag',
            'f_frsize', 'f_namemax'))

    def unlink(self, path):
        print( "unlink(self, %s)" %(path) )
        raise FuseOSError(errno.EACCES)

    def symlink(self, name, target):
        print( "symlink(self, %s, %s)" % (name, target) )
        raise FuseOSError(errno.EACCES)

    def rename(self, old, new):
        print( "rename(self, %s, %s)" % (old, new) )
        raise FuseOSError(errno.EACCES)

    def link(self, target, name):
        print("link(self, %s, %s)" % (target, name) )
        raise FuseOSError(errno.EACCES)

    def utimens(self, path, times=None):
        print( "utimens(self, %s, %s)" % (path, times) )
        return os.utime(self._full_path(path), times)

    # File methods
    # ============
    def open(self, path, flags):
        print( "open(self, %s, %s)" % (path, flags) )
        p = None
        for sfs in self.metadata:
            if( sfs.trace( path, False ) ):
                p = sfs.name + path
        full_path = p

        if(full_path == None):
            raise FuseOSError(ENOENT)
        return os.open(full_path, flags)

    def create(self, path, mode, fi=None):
        print("create(self, %s, %s, %s)" % (path, mode, fi) )
        return os.open(self.writeable + "/" + path, os.O_WRONLY | os.O_CREAT, mode)
        # raise FuseOSError(errno.EACCES)

    def read(self, path, length, offset, fh):
        print("read(self, %s, %s, %s, %s)" % (path, length, offset, fh) )
        os.lseek(fh, offset, os.SEEK_SET)
        return os.read(fh, length)

    def write(self, path, buf, offset, fh):
        print("write(self, %s, %s, %s, %s)" % (path, buf, offset, fh) )
        raise FuseOSError(errno.EACCES)

    def truncate(self, path, length, fh=None):
        print("truncate(self, %s, %s, %s" % (path, length, fh) )
        full_path = self._full_path(path)
        with open(full_path, 'r+') as f:
            f.truncate(length)

    def flush(self, path, fh):
        print("flush(self, %s, %s)" % (path, fh) )
        return os.fsync(fh)

    def release(self, path, fh):
        print("release(self, %s, %s)" % (path, fh) )
        return os.close(fh)

    def fsync(self, path, fdatasync, fh):
        print( "fsync(self, %s, %s, %s)" % (path, fdatasync, fh) )
        return self.flush(path, fh)
