package ibu

import (
	"errors"
	"io/ioutil"
	"os"
	"path/filepath"
	"syscall"
)

// Link all files in abspath to mirpath and recurse
func Link(abspath string, mirpath string) error {

	// We verify that the directory provided to us in abspath actually exists
	if _, err := os.Stat(abspath); os.IsNotExist(err) {
		return errors.New("abspath is bad")
	}

	// We verify that the remote path exists
	if _, err := os.Stat(mirpath); os.IsNotExist(err) {
		// Create mirpath
		s, _ := os.Stat(abspath)
		os.Mkdir(mirpath, 0755)
		os.Chmod(mirpath, s.Mode())

		// !TODO! Not portable
		uid := s.Sys().(*syscall.Stat_t).Uid
		gid := s.Sys().(*syscall.Stat_t).Gid
		os.Chown(mirpath, int(uid), int(gid))

	}

	// Link/Recurse as needed
	files, _ := ioutil.ReadDir(abspath)
	for _, f := range files {
		src, _ := filepath.Abs(abspath + "/" + f.Name())
		dest, _ := filepath.Abs(mirpath + "/" + f.Name())

		switch mode := f.Mode(); {
		case mode.IsDir():
			err := Link(src, dest)
			if err != nil {
				return err
			}
		case mode.IsRegular():
			os.Symlink(src, dest)
		}
	}

	return nil
}
