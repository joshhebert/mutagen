package ibu

import (
	"archive/tar"
	"compress/gzip"
	"fmt"
	"io"
	"os"
)

type memArchive struct {
	reader *tar.Reader
}


func overwrite(mpath string) (*os.File, error) {
	f, err := os.OpenFile(mpath, os.O_RDWR|os.O_TRUNC, 0777)
	if err != nil {
		f, err = os.Create(mpath)
		if err != nil {
			return f, err
		}
	}
	return f, nil
}


func NewArchiveReader(src string) *memArchive {
	f, err := os.Open(src)

	if err != nil {
		fmt.Println(err)
		os.Exit(1)
	}
	defer f.Close()

	gzf, err := gzip.NewReader(f)
	if err != nil {
		fmt.Println(err)
		os.Exit(1)
	}

	tarReader := tar.NewReader(gzf)

	return &memArchive{reader: tarReader};
}

func (archive *memArchive) Decompress(to string) {
	for {
		header, err := archive.reader.Next()

		if err == io.EOF {
			break
		}

		if err != nil {
			fmt.Println(err)
			os.Exit(1)
		}

		// TODO deal with symlinks
		switch header.Typeflag {
		case tar.TypeDir:
			if err := os.MkdirAll(to+"/"+header.Name, os.FileMode(header.Mode)); err != nil {
				panic(err)
			}

		case tar.TypeReg:
			ow, err := overwrite(to + "/" + header.Name)
			defer ow.Close()
			if err != nil {
				panic(err)
			}
			if _, err := io.Copy(ow, archive.reader); err != nil {
				panic(err)
			}
			os.Chmod(ow.Name(), os.FileMode(header.Mode))

		default:
			fmt.Printf("Can't: %c, %s\n", header.Typeflag, header.Name)
		}
	}
}
