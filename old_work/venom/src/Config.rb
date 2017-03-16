require "singleton"

class Config
    include Singleton
    attr_reader :cache_dir
    attr_reader :install_dir
    attr_reader :work_dir
    def initialize()
        @install_dir = "./fake_fs/repo"
        @work_dir = "./fake_fs/unpack"
        @cache_dir = "./fake_fs/cache"
    end

end 
