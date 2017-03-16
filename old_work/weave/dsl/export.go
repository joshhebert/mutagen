package dsl

// Package contains
type Package struct {
	Name    string
	Version string
	Arch    string
}

/* ManifestResolver
 * We expect any manifest returned by this to be a complete copy that can be
 * freely mutated without affecting any other part of the program
 */
type ManifestResolver interface {
	getManifest(string, string) (*manifest, error)
}

// In order to allow us to prioritize one type of resolver over another,
// we implement multiManifestResolver, which tries all the ManifestResolvers
// in its queue until it gets one that isn't nil
type MultiManifestResolver struct {
	Resolvers []ManifestResolver
}

// Queries a remote server to resolve the given manifest
type ManifestNetResolver struct{}

/* ManifestFSResolver
 *
 * Attempts to resolve a name/version pair by looking at manifest files stored
 * locally on the system,
 */
type ManifestFSResolver struct{}

// Given a list of packages, resolve the entire tree for them
func Resolve(packages []Package, mfr ManifestResolver) []Package {
	cps := []concretePackage{}
	for _, p := range packages {
		cps = append(cps, concretePackage{name: p.Name, version: p.Version})
	}

	tracker, _ := newTracker(cps, mfr)

	// Consolidate into a slice
	pkgList, _ := tracker.flattenDepGraph()

	outputPackages := []Package{}
	for _, e := range pkgList {
		outputPackages = append(outputPackages, Package{Name: e.name, Version: e.version})

	}

	return outputPackages

}
