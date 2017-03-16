package dsl

import (
	"fmt"
	"math/rand"
	"runtime"
	"sync"
	"testing"
	"time"
)

/* Faked manifest resolver to be used for testing. */
type manifestTestResolver struct {
	manifests map[concretePackage]manifest
	mutex     *sync.Mutex
}

func (mf manifestTestResolver) getManifest(name string, version string) (*manifest, error) {
	mf.mutex.Lock()
	defer mf.mutex.Unlock()

	// We sleep for some random amount of time to stir things up
	// Sleep for somewhere between 0 and 50 ms
	delay := false
	if delay {
		//time.Sleep( time.Millisecond * time.Duration(rand.Int31n( 50 ) ) )
		time.Sleep(time.Millisecond * time.Duration(1000))
	}
	var target concretePackage
	target.name = name
	target.version = version

	var m manifest
	m = mf.manifests[target]

	// Do a deep copy of the manifest
	var mCopy manifest
	mCopy.target = m.target
	for _, dep := range m.require {
		mCopy.require = append(mCopy.require, loosePackage{dep.name, dep.minVersion, dep.maxVersion})
	}
	return &mCopy, nil
}

func TestMutability(t *testing.T) {
	var mfp manifestTestResolver

	mfp.mutex = &sync.Mutex{}
	mfp.manifests = make(map[concretePackage]manifest)
	mfp.manifests[concretePackage{name: "mypackage", version: "1.0"}] = manifest{
		target: concretePackage{name: "mypackage", version: "1.0"},
		require: []loosePackage{
			loosePackage{
				name:       "library1",
				minVersion: "1.0",
				maxVersion: "1.0",
			},
		},
	}

	mf, err := mfp.getManifest("mypackage", "1.0")
	if err != nil {
		// Who cares?
	}

	mf.require[0].name = "Kappa"

	mf2, err2 := mfp.getManifest("mypackage", "1.0")
	if err2 != nil {
		// Who cares?
	}
	if mf2.require[0].name == "Kappa" {
		t.Fail()
	}
}

/*
 * Test that when a dependency is update so that it no longer requires a
 * particular child, that child is pruned away
 */
func TestMultipleChildren(t *testing.T) {
	// Test configuration
	runtime.GOMAXPROCS(10)
	rand.Seed(time.Now().UTC().UnixNano())
	//// End test configuration

	// Build a fake registry of packages
	var mfp manifestTestResolver
	mfp.mutex = &sync.Mutex{}
	mfp.manifests = make(map[concretePackage]manifest)
	mfp.manifests[concretePackage{name: "mypackage", version: "1.0"}] = manifest{
		target: concretePackage{name: "mypackage", version: "1.0"},
		require: []loosePackage{
			loosePackage{
				name:       "library1",
				minVersion: "1.0",
				maxVersion: "1.0",
			},
			loosePackage{
				name:       "library2",
				minVersion: "1.0",
				maxVersion: "1.0",
			},
			loosePackage{
				name:       "library3",
				minVersion: "1.0",
				maxVersion: "1.0",
			},
		},
	}
	mfp.manifests[concretePackage{name: "library1", version: "1.0"}] = manifest{
		target: concretePackage{name: "library1", version: "1.0"},
		require: []loosePackage{
			loosePackage{
				name:       "child1",
				minVersion: "1.0",
				maxVersion: "1.0",
			},
			loosePackage{
				name:       "child2",
				minVersion: "1.0",
				maxVersion: "1.0",
			},
			loosePackage{
				name:       "child3",
				minVersion: "1.0",
				maxVersion: "1.0",
			},
		},
	}
	mfp.manifests[concretePackage{name: "library2", version: "1.0"}] = manifest{
		target:  concretePackage{name: "library2", version: "1.0"},
		require: []loosePackage{},
	}
	mfp.manifests[concretePackage{name: "library3", version: "1.0"}] = manifest{
		target:  concretePackage{name: "library3", version: "1.0"},
		require: []loosePackage{},
	}
	mfp.manifests[concretePackage{name: "child1", version: "1.0"}] = manifest{
		target:  concretePackage{name: "child1", version: "1.0"},
		require: []loosePackage{},
	}
	mfp.manifests[concretePackage{name: "child2", version: "1.0"}] = manifest{
		target:  concretePackage{name: "child2", version: "1.0"},
		require: []loosePackage{},
	}
	mfp.manifests[concretePackage{name: "child3", version: "1.0"}] = manifest{
		target:  concretePackage{name: "child3", version: "1.0"},
		require: []loosePackage{},
	}
	// Build our tracker
	cp := concretePackage{
		name:    "mypackage",
		version: "1.0",
	}
	tracker, err := newTracker([]concretePackage{cp}, mfp)
	if err != nil {
		t.Fail()
	}

	// Consolidate into a slice
	pkgList, err := tracker.flattenDepGraph()
	if err != nil {
		t.Fail()
	}

	// Put in a form that's easy to test
	res := make(map[string]concretePackage)
	for _, e := range pkgList {
		res[e.name] = *e
	}

	if (res["mypackage"] != concretePackage{name: "mypackage", version: "1.0"}) {
		t.Fail()
	}
	if (res["library1"] != concretePackage{name: "library1", version: "1.0"}) {
		t.Fail()
	}
	if (res["library2"] != concretePackage{name: "library2", version: "1.0"}) {
		t.Fail()
	}
	if (res["library3"] != concretePackage{name: "library3", version: "1.0"}) {
		t.Fail()
	}
	if (res["child1"] != concretePackage{name: "child1", version: "1.0"}) {
		t.Fail()
	}
	if (res["child2"] != concretePackage{name: "child2", version: "1.0"}) {
		t.Fail()
	}
	if (res["child3"] != concretePackage{name: "child3", version: "1.0"}) {
		t.Fail()
	}
}

/*
 * Test that when a dependency is update so that it no longer requires a
 * particular child, that child is pruned away
 */
func TestPrune(t *testing.T) {
	// Test configuration
	runtime.GOMAXPROCS(10)
	rand.Seed(time.Now().UTC().UnixNano())
	//// End test configuration
	/*	 	 /-- library1@2.0 -- prunee -- pruneechild
	mypackage
			 \-- pruner

			 /-- library1@1.0
	mypackage
			 \-- pruner
	*/

	// Build a fake registry of packages
	var mfp manifestTestResolver
	mfp.mutex = &sync.Mutex{}
	mfp.manifests = make(map[concretePackage]manifest)
	mfp.manifests[concretePackage{name: "mypackage", version: "1.0"}] = manifest{
		target: concretePackage{name: "mypackage", version: "1.0"},
		require: []loosePackage{
			loosePackage{
				name:       "library1",
				minVersion: "1.0",
				maxVersion: "2.0",
			},
			loosePackage{
				name:       "pruner",
				minVersion: "1.0",
				maxVersion: "1.0",
			},
		},
	}
	mfp.manifests[concretePackage{name: "library1", version: "2.0"}] = manifest{
		target: concretePackage{name: "library1", version: "2.0"},
		require: []loosePackage{
			loosePackage{
				name:       "prunee",
				minVersion: "1.0",
				maxVersion: "1.0",
			},
		},
	}
	mfp.manifests[concretePackage{name: "library1", version: "1.0"}] = manifest{
		target:  concretePackage{name: "library1", version: "1.0"},
		require: []loosePackage{},
	}
	mfp.manifests[concretePackage{name: "prunee", version: "1.0"}] = manifest{
		target: concretePackage{name: "prunee", version: "1.0"},
		require: []loosePackage{
			loosePackage{
				name:       "pruneechild",
				minVersion: "1.0",
				maxVersion: "1.0",
			},
		},
	}
	mfp.manifests[concretePackage{name: "pruneechild", version: "1.0"}] = manifest{
		target:  concretePackage{name: "pruneechild", version: "1.0"},
		require: []loosePackage{},
	}
	mfp.manifests[concretePackage{name: "pruner", version: "1.0"}] = manifest{
		target: concretePackage{name: "pruner", version: "1.0"},
		require: []loosePackage{
			loosePackage{
				name:       "library1",
				minVersion: "1.0",
				maxVersion: "1.0",
			},
		},
	}
	// Build our tracker
	cp := concretePackage{
		name:    "mypackage",
		version: "1.0",
	}
	tracker, err := newTracker([]concretePackage{cp}, &mfp)
	if err != nil {
		t.Fail()
	}

	// Consolidate into a slice
	pkgList, err := tracker.flattenDepGraph()
	if err != nil {
		t.Fail()
	}

	// Put in a form that's easy to test
	res := make(map[string]concretePackage)
	for _, e := range pkgList {
		res[e.name] = *e
	}

	if (res["mypackage"] != concretePackage{name: "mypackage", version: "1.0"}) {
		t.Fail()
	}
	if (res["library1"] != concretePackage{name: "library1", version: "1.0"}) {
		t.Fail()
	}
	if (res["pruner"] != concretePackage{name: "pruner", version: "1.0"}) {
		t.Fail()
	}
	if (res["prunee"] != concretePackage{}) {
		t.Fail()
	}
	if (res["pruneechild"] != concretePackage{}) {
		t.Fail()
	}
}

/*
 * Stupidly large dependency tree with all sorts of obnoxious constructs in it.
 * Written to be a stress test/catch-all for testing the tracker. Mainly used
 * to ensure that there are no problems with the concurrency model
 *
 * Work in progress
 */
func TestTheGauntlet(t *testing.T) {
	// Test configuration
	runtime.GOMAXPROCS(10)
	rand.Seed(time.Now().UTC().UnixNano())
	//// End test configuration

	// Build a fake registry of packages
	var mfp manifestTestResolver
	mfp.mutex = &sync.Mutex{}
	mfp.manifests = make(map[concretePackage]manifest)
	mfp.manifests[concretePackage{name: "mypackage", version: "4.5"}] = manifest{
		target: concretePackage{name: "mypackage", version: "4.5"},
		require: []loosePackage{
			loosePackage{
				name:       "library1",
				minVersion: "_",
				maxVersion: "1.5",
			},
			loosePackage{
				name:       "library2",
				minVersion: "2.0",
				maxVersion: "3.0",
			},
			loosePackage{
				name:       "library6",
				minVersion: "5.0",
				maxVersion: "9.0",
			},
		},
	}
	mfp.manifests[concretePackage{name: "library1", version: "1.5"}] = manifest{
		target: concretePackage{name: "library1", version: "1.5"},
		require: []loosePackage{
			loosePackage{
				name:       "library4",
				minVersion: "2.0",
				maxVersion: "2.0",
			},
		},
	}
	mfp.manifests[concretePackage{name: "library1", version: "1.0"}] = manifest{
		target: concretePackage{name: "library1", version: "1.0"},
		require: []loosePackage{
			loosePackage{
				name:       "library4",
				minVersion: "1.0",
				maxVersion: "1.0",
			},
		},
	}
	mfp.manifests[concretePackage{name: "library2", version: "3.0"}] = manifest{
		target: concretePackage{name: "library2", version: "3.0"},
		require: []loosePackage{
			loosePackage{
				name:       "library3",
				minVersion: "2.0",
				maxVersion: "4.0",
			},
			loosePackage{
				name:       "library4",
				minVersion: "1.0",
				maxVersion: "3.0",
			},
		},
	}

	mfp.manifests[concretePackage{name: "library3", version: "4.0"}] = manifest{
		target: concretePackage{name: "library3", version: "4.0"},
		require: []loosePackage{
			loosePackage{
				name:       "library1",
				minVersion: "0.8",
				maxVersion: "1.0",
			},
		},
	}

	mfp.manifests[concretePackage{name: "library4", version: "1.0"}] = manifest{
		target:  concretePackage{name: "library4", version: "1.0"},
		require: []loosePackage{},
	}
	mfp.manifests[concretePackage{name: "library4", version: "2.0"}] = manifest{
		target: concretePackage{name: "library4", version: "2.0"},
		require: []loosePackage{
			loosePackage{
				name:       "library5",
				minVersion: "1.0",
				maxVersion: "1.0",
			},
		},
	}

	// This should die of neglect
	mfp.manifests[concretePackage{name: "library5", version: "1.0"}] = manifest{
		target: concretePackage{name: "library5", version: "1.0"},
		require: []loosePackage{
			loosePackage{
				name:       "library9",
				minVersion: "1.0",
				maxVersion: "1.0",
			},
			loosePackage{
				name:       "library6",
				minVersion: "4.0",
				maxVersion: "5.0",
			},
		},
	}

	mfp.manifests[concretePackage{name: "library6", version: "5.0"}] = manifest{
		target: concretePackage{name: "library6", version: "5.0"},
		require: []loosePackage{
			loosePackage{
				name:       "library7",
				minVersion: "1.0",
				maxVersion: "1.0",
			},
			loosePackage{
				name:       "library8",
				minVersion: "1.0",
				maxVersion: "1.0",
			},
		},
	}
	mfp.manifests[concretePackage{name: "library6", version: "7.0"}] = manifest{
		target: concretePackage{name: "library6", version: "7.0"},
		require: []loosePackage{
			loosePackage{
				name:       "library7",
				minVersion: "1.0",
				maxVersion: "1.0",
			},
			loosePackage{
				name:       "library8",
				minVersion: "1.0",
				maxVersion: "1.0",
			},
		},
	}
	mfp.manifests[concretePackage{name: "library6", version: "9.0"}] = manifest{
		target: concretePackage{name: "library6", version: "9.0"},
		require: []loosePackage{
			loosePackage{
				name:       "library7",
				minVersion: "1.0",
				maxVersion: "1.0",
			},
			loosePackage{
				name:       "library8",
				minVersion: "1.0",
				maxVersion: "1.0",
			},
		},
	}
	mfp.manifests[concretePackage{name: "library7", version: "1.0"}] = manifest{
		target:  concretePackage{name: "library7", version: "1.0"},
		require: []loosePackage{},
	}
	mfp.manifests[concretePackage{name: "library8", version: "1.0"}] = manifest{
		target:  concretePackage{name: "library8", version: "1.0"},
		require: []loosePackage{},
	}
	mfp.manifests[concretePackage{name: "library9", version: "1.0"}] = manifest{
		target:  concretePackage{name: "library9", version: "1.0"},
		require: []loosePackage{},
	}

	// Build our tracker
	cp := concretePackage{
		name:    "mypackage",
		version: "4.5",
	}
	tracker, err := newTracker([]concretePackage{cp}, &mfp)
	if err != nil {
		t.Fail()
	}

	// Consolidate into a slice
	pkgList, err := tracker.flattenDepGraph()
	if err != nil {
		t.Fail()
	}

	// Put in a form that's easy to test
	res := make(map[string]concretePackage)
	for _, e := range pkgList {
		res[e.name] = *e
	}

	if (res["mypackage"] != concretePackage{name: "mypackage", version: "4.5"}) {
		fmt.Printf("Expecting %s-%s, but got %s-%s\n",
			"mypackage", "4.5",
			res["mypackage"].name, res["mypackage"].version)
		t.Fail()
	}
	if (res["library1"] != concretePackage{name: "library1", version: "1.0"}) {
		fmt.Printf("Expecting %s-%s, but got %s-%s\n",
			"library1", "1.0",
			res["library1"].name, res["library1"].version)
		t.Fail()
	}
	if (res["library2"] != concretePackage{name: "library2", version: "3.0"}) {
		fmt.Printf("Expecting %s-%s, but got %s-%s\n",
			"library2", "3.0",
			res["library2"].name, res["library2"].version)
		t.Fail()
	}
	if (res["library3"] != concretePackage{name: "library3", version: "4.0"}) {
		fmt.Printf("Expecting %s-%s, but got %s-%s\n",
			"library3", "4.0",
			res["library3"].name, res["library3"].version)
		t.Fail()
	}
	if (res["library4"] != concretePackage{name: "library4", version: "1.0"}) {
		fmt.Printf("Expecting %s-%s, but got %s-%s\n",
			"library4", "1.0",
			res["library4"].name, res["library4"].version)
		t.Fail()
	}
	if (res["library5"] != concretePackage{}) {
		fmt.Printf("Expecting %s to be empty, but got %s-%s\n",
			"library5",
			res["library5"].name, res["library5"].version)
		t.Fail()
	}
	if (res["library6"] != concretePackage{name: "library6", version: "9.0"}) {
		fmt.Printf("Expecting %s-%s, but got %s-%s\n",
			"library6", "9.0",
			res["library6"].name, res["library6"].version)
		t.Fail()
	}
	if (res["library7"] != concretePackage{name: "library7", version: "1.0"}) {
		fmt.Printf("Expecting %s-%s, but got %s-%s\n",
			"library7", "1.0",
			res["library7"].name, res["library7"].version)
		t.Fail()
	}

	if (res["library8"] != concretePackage{name: "library8", version: "1.0"}) {
		fmt.Printf("Expecting %s-%s, but got %s-%s\n",
			"library8", "1.0",
			res["library8"].name, res["library8"].version)
		t.Fail()
	}
	if (res["library9"] != concretePackage{}) {
		fmt.Printf("Expecting %s to be empty, but got %s-%s\n",
			"library9",
			res["library9"].name, res["library9"].version)
		t.Fail()
	}
}

func BenchmarkStandard(b *testing.B) {
	// Test configuration
	runtime.GOMAXPROCS(10)
	//rand.Seed(time.Now().UTC().UnixNano())
	//// End test configuration

	// Build a fake registry of packages
	var mfp manifestTestResolver
	mfp.mutex = &sync.Mutex{}
	mfp.manifests = make(map[concretePackage]manifest)
	mfp.manifests[concretePackage{name: "mypackage", version: "4.5"}] = manifest{
		target: concretePackage{name: "mypackage", version: "4.5"},
		require: []loosePackage{
			loosePackage{
				name:       "library1",
				minVersion: "_",
				maxVersion: "1.5",
			},
			loosePackage{
				name:       "library2",
				minVersion: "2.0",
				maxVersion: "3.0",
			},
			loosePackage{
				name:       "library6",
				minVersion: "5.0",
				maxVersion: "9.0",
			},
		},
	}
	mfp.manifests[concretePackage{name: "library1", version: "1.5"}] = manifest{
		target: concretePackage{name: "library1", version: "1.5"},
		require: []loosePackage{
			loosePackage{
				name:       "library4",
				minVersion: "2.0",
				maxVersion: "2.0",
			},
		},
	}
	mfp.manifests[concretePackage{name: "library1", version: "1.0"}] = manifest{
		target: concretePackage{name: "library1", version: "1.0"},
		require: []loosePackage{
			loosePackage{
				name:       "library4",
				minVersion: "1.0",
				maxVersion: "1.0",
			},
		},
	}
	mfp.manifests[concretePackage{name: "library2", version: "3.0"}] = manifest{
		target: concretePackage{name: "library2", version: "3.0"},
		require: []loosePackage{
			loosePackage{
				name:       "library3",
				minVersion: "2.0",
				maxVersion: "4.0",
			},
			loosePackage{
				name:       "library4",
				minVersion: "1.0",
				maxVersion: "3.0",
			},
		},
	}

	mfp.manifests[concretePackage{name: "library3", version: "4.0"}] = manifest{
		target: concretePackage{name: "library3", version: "4.0"},
		require: []loosePackage{
			loosePackage{
				name:       "library1",
				minVersion: "0.8",
				maxVersion: "1.0",
			},
		},
	}

	mfp.manifests[concretePackage{name: "library4", version: "1.0"}] = manifest{
		target:  concretePackage{name: "library4", version: "1.0"},
		require: []loosePackage{},
	}
	mfp.manifests[concretePackage{name: "library4", version: "2.0"}] = manifest{
		target: concretePackage{name: "library4", version: "2.0"},
		require: []loosePackage{
			loosePackage{
				name:       "library5",
				minVersion: "1.0",
				maxVersion: "1.0",
			},
		},
	}

	// This should die of neglect
	mfp.manifests[concretePackage{name: "library5", version: "1.0"}] = manifest{
		target: concretePackage{name: "library5", version: "1.0"},
		require: []loosePackage{
			loosePackage{
				name:       "library9",
				minVersion: "1.0",
				maxVersion: "1.0",
			},
			loosePackage{
				name:       "library6",
				minVersion: "4.0",
				maxVersion: "5.0",
			},
		},
	}

	mfp.manifests[concretePackage{name: "library6", version: "5.0"}] = manifest{
		target: concretePackage{name: "library6", version: "5.0"},
		require: []loosePackage{
			loosePackage{
				name:       "library7",
				minVersion: "1.0",
				maxVersion: "1.0",
			},
			loosePackage{
				name:       "library8",
				minVersion: "1.0",
				maxVersion: "1.0",
			},
		},
	}
	mfp.manifests[concretePackage{name: "library6", version: "7.0"}] = manifest{
		target: concretePackage{name: "library6", version: "7.0"},
		require: []loosePackage{
			loosePackage{
				name:       "library7",
				minVersion: "1.0",
				maxVersion: "1.0",
			},
			loosePackage{
				name:       "library8",
				minVersion: "1.0",
				maxVersion: "1.0",
			},
		},
	}
	mfp.manifests[concretePackage{name: "library6", version: "9.0"}] = manifest{
		target: concretePackage{name: "library6", version: "9.0"},
		require: []loosePackage{
			loosePackage{
				name:       "library7",
				minVersion: "1.0",
				maxVersion: "1.0",
			},
			loosePackage{
				name:       "library8",
				minVersion: "1.0",
				maxVersion: "1.0",
			},
		},
	}
	mfp.manifests[concretePackage{name: "library7", version: "1.0"}] = manifest{
		target:  concretePackage{name: "library7", version: "1.0"},
		require: []loosePackage{},
	}
	mfp.manifests[concretePackage{name: "library8", version: "1.0"}] = manifest{
		target:  concretePackage{name: "library8", version: "1.0"},
		require: []loosePackage{},
	}
	mfp.manifests[concretePackage{name: "library9", version: "1.0"}] = manifest{
		target:  concretePackage{name: "library9", version: "1.0"},
		require: []loosePackage{},
	}

	// Build our tracker
	cp := concretePackage{
		name:    "mypackage",
		version: "4.5",
	}

	b.ResetTimer()

	for i := 0; i < b.N; i++ {
		_,_ = newTracker([]concretePackage{cp}, &mfp)
    }
}
