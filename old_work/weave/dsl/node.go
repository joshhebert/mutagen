package dsl

import (
	"sync"
)

type depNode struct {
	tracker       *tracker
	updateChannel chan nodeRuleUpdate
	name          string
	targetVersion string
	rules         []rule
}

func (node *depNode) mfQuery(name string, version string) manifest {

	cb := make(chan *manifest)
	q := manifestQuery{cb, concretePackage{name, version}}
	node.tracker.mfChannel <- q
	mf := <-cb
	return *mf
}

/*
 * Cause a node to go "live", i.e. be accessible to other nodes.
 * This is a node's main loop, which will persist until either
 * the graph is settled, or it dies of neglect (i.e. it isn't referenced
 * by any other node.)
 */
func (node *depNode) live() {
	//fmt.Printf( "%s: alive\n", node.Name );
	for {
		//fmt.Printf( "%s: Waiting for updates...\n", node.Name );
		update := <-node.updateChannel
		update.apply(node)
		//fmt.Printf( "%s: Rules updated\n", node.Name );

		//fmt.Printf( "%s: I'm done with my update\n", node.Name );

		if len(node.rules) == 0 {
			// Acquire the manifest of this node
			mf := node.mfQuery(node.name, node.targetVersion)

			//fmt.Printf( "%s: Cleaning children\n", node.Name );
			deleteBlocker := &sync.WaitGroup{}
			for _, child := range mf.require {

				cbChild := make(chan *depNode)
				qChild := refQuery{cbChild, child.name}
				node.tracker.refChannel <- qChild
				c := <-cbChild
				if c != nil {
					deleteBlocker.Add(1)
					//fmt.Printf( "%s: Sending clear rule op to %s\n", node.Name, c.Name );
					c.updateChannel <- nodeRuleUpdate{
						owner:   node.name,
						blocker: deleteBlocker,
					}
				}
			}
			deleteBlocker.Wait()

		}

		// Report the update as done
		update.blocker.Done()
	}
}

func (node *depNode) validate(newVersion string) error {
	/*
	 * Compare the old and new dependencies and do one of four things:
	 *	1. If they share the same name and min/max versions, ignore
	 *	2. Same name, but different version requirements, add to update
	 *  3. Not found in old manifest, add to update
	 *  4. Not found in new manifest, add to delete
	 */
	var update []*loosePackage
	var delete []*loosePackage

	// Resolve manifests
	mfOld := node.mfQuery(node.name, node.targetVersion)
	mfNew := node.mfQuery(node.name, newVersion)

	// Populate arrays
	// It would be good to clean this up
	for _, newChild := range mfNew.require {
		found := false
		for index, oldChild := range mfOld.require {
			if oldChild.name == newChild.name {
				//fmt.Printf( "%s: %s update\n", n.Name, newChild.Name );
				mfOld.require = append(mfOld.require[:index], mfOld.require[index+1:]...)
				if oldChild.maxVersion == newChild.maxVersion &&
					oldChild.minVersion == newChild.minVersion {
					found = true
					break
				} else {
					found = true
					update = append(update, &newChild)
					break
				}
			}
		}
		if !found {
			// Reference to a variable created with a foreach causes bad
			// things to happen
			// Ew
			l := newChild
			update = append(update, &l)
		}
	}
	for _, remaining := range mfOld.require {
		delete = append(delete, &remaining)
	}

	// Set this node to the new target version
	node.targetVersion = newVersion

	// Notify deleted nodes holding rules from this node that they're no
	// longer valid
	validateBlocker := &sync.WaitGroup{}

	for _, child := range delete {
		// Send request to tracker
		cbChild := make(chan *depNode)
		qChild := refQuery{cbChild, child.name}
		node.tracker.refChannel <- qChild
		c := <-cbChild

		if c != nil {
			validateBlocker.Add(1)
			c.updateChannel <- nodeRuleUpdate{
				owner:   node.name,
				blocker: validateBlocker,
			}
		}
	}

	// Push updates to all nodes that need them
	for _, child := range update {
		//fmt.Printf( "%s: Dispatching update to %s\n", n.Name, child.Name );
		update := rule{
			owner:      node,
			minVersion: child.minVersion,
			maxVersion: child.maxVersion,
		}

		validateBlocker.Add(1)
		node.tracker.upsertNode(child.name, update, validateBlocker)
	}

	validateBlocker.Wait()
	return nil
}
