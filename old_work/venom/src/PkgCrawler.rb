require 'digest'
require "singleton"
require "xz"
require "archive/tar/minitar"
include Archive::Tar
require "fileutils"

require_relative "./testdata.rb"
require_relative "./Config.rb"
# Given the name and version of a package that has been loaded to the tree,
# but not necessarily linked, trace out its file hierarchy and return a packages
# Additionally, once we have traced a package, we assume packages of the same version
# number will not change in structure, so we cache the trace off as a JSON document.
# In this way, when relinking a previously unlinked package, we don't need to open it
# up again to trace its structure. Rather, we can just check to see if the package dir
# is installed, and if it is, consult the cache to ficgure out how to link it.

class PkgCrawler
    include Singleton
    def initialize( )
        @pkg_cache = {}
    end


    # This function is used to resolve an already installed package. It first checks for a
    # cache of the package metadata. Most of the time, it should be able to get this. However,
    # if for whatever reason it cannot, it will go and traverse the package's file system and
    # map it into metadata. It will then store this cache file
    def resolve( name, version )
        # It is possible, and almost inevitable, that we can install two packages that must oscilate
        # at the same time. As a result, we will be relinking a package that has not been installed yet,
        # but will be on the next commit. In this situation, we need keep a cache of packages mapped in
        # this session, so that when this occurs, we can resolve it with no problems. In this way, we
        # maintain an instance variable hash that is added to whenever map() is called
        cached = @pkg_cache[ "%s-%s" % [ name, version ] ]
        if( cached != nil )
            return cached
        end

        # Past this, we do not want to proceed if the package is not present in the install
        # dir

        # We ensure that this package is in fact installed
        if( !File.directory?( Config.instance.install_dir + "/" + name + "/" + version ) )
            puts "%s-%s is not installed!" % [ name, version ]
            return nil
        end

        # If the cache file exists and is in fact a file, slurp it up and use that
        # I'd love to do some digest check here to:
        #   1. Ensure that the cache file is intact and valid
        #   2. The directory installed matches the one that this cache file was generated from

        # First, we check the cache to see if we've resolved this package before
        # Figure out where the cache file would be if it existed
        cache_file = Config.instance.cache_dir + "/" + name + "-" + version + ".json"
        if( File.file?( cache_file ) )
            file = File.new( cache_file, "r" )
            buff = ""
            while (line = file.gets)
                buff += line
            end
            file.close( )
            pkg = JSON.parse( buff )
            return pkg
        end

        # If we haven't, we'll need to do a full trace of the package

        # It may be better to store all of our package file in
        #   $(install_dir)/$(name)/$(version)/pkg_files
        # and store the manifest next to pkg_files
        # Therefore, when we need to do this, we can just consult the manifest for
        # data such as dependencies
        pkg = map( Config.instance.install_dir + "/" + name + "/" + version )
        file = File.new( cache_file, "w" )

        # Write metadata
        file.write( pkg.to_json( ) )

        file.close( )

        return pkg
    end


    def traverse( dir, relapath )
        pkg_files = Node.new( { } )

        Dir.foreach( dir + relapath ) do |item|
            next if item == '.' or item == '..'
            # If this is a directory, we recurse
            full_path = dir + relapath + "/" + item
            if( File.directory?( full_path ) )
                pkg_files.children[ relapath + "/" + item ] = traverse( dir, relapath + "/" + item )
            elsif( File.file?( full_path ) )
                pkg_files.children[ relapath + "/" + item ] = Node.new( { } )
            else
                puts "Error: Failed to traverse package"
            end
        end
        return pkg_files
    end

    def unpack( pkg_path )
        work_dir = Config.instance.work_dir
        # We should be checking if the file exists...
        # TODO

        # Extract our package to the work dir
        XZ.decompress_file( pkg_path, work_dir + "/tmp.tar")
        Minitar.unpack( work_dir + '/tmp.tar', work_dir )
        FileUtils.rm( work_dir + "/tmp.tar" )
        return File.basename( pkg_path ).split( ".txz" )[ 0 ]
    end

    # pkg_path is the path to the txz package file
    # This is just a tar.xz file
    def map( pkg_path )
        # This is the package we will return
        p = nil

        # This will give us a list of all the files the package will install
        pkg_files = traverse( pkg_path + "/pkg_files", "" )

        # We then read the manifest for other data that we need
        mf = pkg_path + "/MANIFEST.json"

        if( File.file?( mf ) )
            file = File.new( mf, "r" )
            buff = ""
            while (line = file.gets)
                buff += line
            end
            file.close( )
            meta = JSON.parse( buff )
            p = Package.new( meta[ "name" ], meta[ "version" ], pkg_files, meta[ "depends" ] )
        end
        puts "PkgCrawler: placing %s-%s in cache" % [ p.name, p.version ]
        name = "%s-%s" % [ p.name, p.version ]
        @pkg_cache[ name ] = p
        return p
    end
end
