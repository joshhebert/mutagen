require "json"


class Package
    attr_reader :name
    attr_reader :version
    attr_reader :package_files
    attr_reader :depends
    def initialize( name, version, package_files, depends )
        @name = name
        @version = version
        @package_files = package_files
        @depends = depends
    
    end
    def to_json( options = nil )
        pkg_hash = {}
        pkg_hash[ 'name' ] = @name
        pkg_hash[ 'version' ] = @version
        
        pkg_hash[ 'files' ] = @package_files
        pkg_hash[ 'depends' ] = @depends
        return JSON.pretty_generate( pkg_hash )

    end
end



class Node
    attr_accessor :children 
    def initialize( children )
        @children = children
    end
    def to_json( options = nil )
        c = {}
        @children.each{ |name, child|
            c[ name ] = child
        }
        return JSON.pretty_generate( c )

    end
end



nano_files = Node.new({
    "/" => Node.new({
        "/usr/" => Node.new({
            "/usr/docs/" => Node.new({
                "/usr/docs/nano/" => Node.new({
                    "/usr/docs/nano/documentation.txt" => Node.new({
                    })
                })
            }),
            "/usr/share/" => Node.new({
                "/usr/share/lib/" => Node.new({
                    "/usr/share/lib/libnano.so" => Node.new({
                    })
                })
            }),
            "/usr/local/" => Node.new({
                "/usr/local/reources.file" => Node.new({
                })
            })
        }),
        "/bin/" => Node.new({
            "/bin/nano" => Node.new({
            })
        })
    })
})

$nano =  Package.new( "nano", "1.3", nano_files, nil ) 
# Test Data
vim_files = Node.new({
    "/" => Node.new({
        "/usr/" => Node.new({
            "/usr/docs/" => Node.new({
                "/usr/docs/vim/" => Node.new({
                    "/usr/docs/vim/documentation.txt" => Node.new({
                    })
                })
            }),
            "/usr/share/" => Node.new({
                "/usr/share/lib/" => Node.new({
                    "/usr/share/lib/libvim.so" => Node.new({
                    })
                })
            })
        }),
        "/bin/" => Node.new({
            "/bin/vim" => Node.new({
            })
        }),
        "/lib/" => Node.new({
            "/lib/libvim2.so" => Node.new({
            })
        })
    })
})

$vim =  Package.new( "vim", "4.5", vim_files, [ "lib1-3", "lib2-3.2", "lib3-5" ] ) 


emacs_files = Node.new({
    "/" => Node.new({
        "/usr/" => Node.new({
            "/usr/docs/" => Node.new({
                "/usr/docs/emacs/" => Node.new({
                    "/usr/docs/emacs/documentation.txt" => Node.new({
                    })
                })
            }),
            "/usr/share/" => Node.new({
                "/usr/share/lib/" => Node.new({
                    "/usr/share/lib/libemacs.so" => Node.new({
                    })
                }),
                "/usr/src/" => Node.new({
                    "/usr/src/emacs.sources" => Node.new({
                    })
                })
            })
        }),
        "/bin/" => Node.new({
            "/bin/emacs" => Node.new({
            })
        })
    })
})

$emacs =  Package.new( "emacs", "1.0", emacs_files, nil ) 
