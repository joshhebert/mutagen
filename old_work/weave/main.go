package main

import (
	"fmt"
	"github.com/joshhebert/weave/dsl"
	"github.com/joshhebert/weave/ibu"
)


func install(pkgPath string, installDir string, env string) {
	ibu.NewArchiveReader(pkgPath)

	/*
	error := ibu.Link(installDir, env)
	if error != nil {
		panic("Fix me!")
	}
	*/
	/*
		files, _ := ioutil.ReadDir(install_dir)
		for _, f := range files {
			link( install_dir, env )
		}
	*/
}

// Stub for testing
func main() {
	//prefix := os.Args[2]

	// Testing package
	fish := dsl.Package{
		Name:    "fish",
		Version: "2.3.1-1",
		Arch:    "x86_64",
	}

	// No servers, so we can only resolve manifests locally
	mfr := dsl.MultiManifestResolver{}
	mfr.Resolvers = append(mfr.Resolvers, dsl.ManifestFSResolver{})

	// Resolve dependencies
	packages := dsl.Resolve([]dsl.Package{fish}, mfr)

	fmt.Println(packages)
	// Acquire package files
	// TODO

	// Install
	//install_dir := fmt.Sprintf( "%s/%s/%s/%s/", prefix, arch, name, version )
	//install(os.Args[1], install_dir, os.Args[3])
}
