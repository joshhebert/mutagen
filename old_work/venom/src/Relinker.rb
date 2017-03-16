require_relative "./Installer.rb"
require_relative "./PkgCrawler.rb"
require_relative "./Database.rb"

class Relinker < Installer
    def initialize( name, version )
        package = PkgCrawler.instance.resolve( name, version )

        @db = Database.instance.get_db( )
        @unbound = []
        @pkg_files = package.package_files
        @pkg_name = package.name
        @pkg_ver = package.version

        @depth = 0
    end
end
