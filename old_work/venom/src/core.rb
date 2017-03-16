require "sqlite3"
require_relative "./Installer.rb"
require_relative "./Uninstaller.rb"
require_relative "./DependencyUtils.rb"
require_relative "./testdata.rb"


#Installer.new( "../test_pkg/vim-4.5.txz" ).integrate( )
#Installer.new( "../test_pkg/emacs-2.0.txz" ).integrate( )
#Installer.new( "../test_pkg/vim-4.6.txz" ).integrate( )
tp1 = Package.new( nil, nil, nil, [
    { "name" => "vim-libs", "minversion" => "4.7", "maxversion" => "4.9" },
    { "name" => "vim-doc", "minversion" => "4.8", "maxversion" => "4.9" }
] )

