package dsl

import (
	"errors"
	"fmt"
	"sync"
)

type refQuery struct {
	callbackChan chan *depNode
	pack         string
}

type manifestQuery struct {
	callbackChan chan *manifest
	pack         concretePackage
}

type tracker struct {
	roots            []*depNode
	manifestResolver *ManifestResolver
	refs             map[string]*depNode
	refChannel       chan refQuery
	mfChannel        chan manifestQuery
	remainingUpdates int
	mutex            *sync.RWMutex
}

func (t *tracker) handleManifestQueries() error {
	//fmt.Printf( "Manifest handler online\n" );
	for {
		q := <-t.mfChannel
		mf, err := (*(t.manifestResolver)).getManifest(q.pack.name, q.pack.version)
		if err != nil {
			return errors.New("Could not resolve package manifest!")
		}

		q.callbackChan <- mf
	}
	//fmt.Printf( "Manifest handler has died\n" );

	return nil
}

func (t *tracker) handleRefQueries() error {
	//fmt.Printf( "Reference handler online\n" );
	for {
		q := <-t.refChannel
		t.mutex.RLock()
		node := t.refs[q.pack]
		t.mutex.RUnlock()
		q.callbackChan <- node
	}
	//fmt.Printf( "Reference handler has died" );
	return nil
}

func newTracker(packs []concretePackage, resolver ManifestResolver) (*tracker, error) {
	instance := &tracker{
		manifestResolver: &resolver,
		refs:             make(map[string]*depNode),
		mutex:            &sync.RWMutex{},
		// TODO									V tsk tsk magic numbers
		mfChannel:        make(chan manifestQuery, 100),
		refChannel:       make(chan refQuery, 100),
		remainingUpdates: 0,
	}

	go instance.handleManifestQueries()
	go instance.handleRefQueries()

	creationBlocker := &sync.WaitGroup{}
	for _, e := range packs {
		initRule := rule{
			owner:      &depNode{},
			minVersion: e.version,
			maxVersion: e.version,
		}

		creationBlocker.Add(1)
		go instance.upsertNode(e.name, initRule, creationBlocker)
	}
	creationBlocker.Wait()

	// Collate our roots
	for _, e := range packs {
		instance.mutex.RLock()
		instance.roots = append(instance.roots, instance.refs[e.name])
		instance.mutex.RUnlock()
	}

	return instance, nil
}

func (t *tracker) upsertNode(name string, initRule rule, wg *sync.WaitGroup) {
	t.mutex.Lock()
	defer t.mutex.Unlock()

	if t.refs[name] != nil {
		node := t.refs[name]
		operation := nodeRuleUpdate{
			owner:   initRule.owner.name,
			rules:   []rule{initRule},
			blocker: wg,
		}
		node.updateChannel <- operation
		return

	}

	// Give the new node a rule
	operation := nodeRuleUpdate{
		owner:   initRule.owner.name,
		rules:   []rule{initRule},
		blocker: wg,
	}

	newNode := &depNode{
		tracker: t,
		//TODO										V tsk tsk magic numbers
		updateChannel: make(chan nodeRuleUpdate, 20),
		name:          name,
		targetVersion: "",
	}
	// If the chan cannot take anymore, we'll need to wait
	newNode.updateChannel <- operation

	t.refs[name] = newNode
	go newNode.live()

	return
}

/*
 * Starting with the root node and using the manifest provider, traverse the
 * network like a tree and return a list of all packages used in the tree,
 * with no duplicates. If this function does not throw an error, the contract
 * is that the list of packages represent a complete runtime with all
 * dependencies satisified in the optimal pattern (i.e. most up-to-date that
 * still meet restrictions)
 */
func (t *tracker) flattenDepGraph() ([]*concretePackage, error) {
	var runtime []*concretePackage

	var cursor func([]*depNode) error

	cursor = func(nodes []*depNode) error {
		for _, node := range nodes {
			runtime = append(runtime, &concretePackage{name: node.name, version: node.targetVersion})

			// Acquire manifest
			mf := node.mfQuery(node.name, node.targetVersion)

			for _, child := range mf.require {
				// Acquire reference
				cbChild := make(chan *depNode)
				qChild := refQuery{cbChild, child.name}
				node.tracker.refChannel <- qChild
				c := <-cbChild

				cursor([]*depNode{c})
			}
		}
		return nil
	}

	res := (cursor(t.roots))
	if res != nil {
		fmt.Println("Errors occured while designing runtime")
		return []*concretePackage{}, nil
	}

	// All we need to do now is strip dups and ensure any dups are exact matches
	var uniq []*concretePackage
	for 0 < len(runtime) {
		// Pop the head off
		target := runtime[0]
		uniq = append(uniq, target)
		runtime = runtime[1:]

		for j := 0; j < len(runtime); {
			if target.name == runtime[j].name {
				if target.version == runtime[j].version {
					// just delete element at j
					runtime = append(runtime[:j], runtime[j+1:]...)
				} else {
					fmt.Println("Error!: Package conflict during flattening")
				}
			} else {
				j++
			}
		}
	}

	return uniq, nil
}
