package dsl

import (
	"errors"
	"fmt"
	"sync"
)

type rule struct {
	owner      *depNode
	minVersion string
	maxVersion string
}

type nodeRuleUpdate struct {
	owner   string
	blocker *sync.WaitGroup
	rules   []rule
}

func (op nodeRuleUpdate) apply(n *depNode) error {
	for index, e := range n.rules {
		if e.owner.name == op.owner {
			//fmt.Printf( "%s: Deleting rule owned by %s (%s-%s)\n", n.Name, e.Owner.Name, e.MinVersion, e.MaxVersion );
			n.rules = append(n.rules[:index], n.rules[index+1:]...)
			break
		}
	}

	// Append new rules (if any)
	for _, r := range op.rules {
		//fmt.Printf( "%s: Applying rule %s-%s\n", n.Name, r.MinVersion, r.MaxVersion );
		n.rules = append(n.rules, r)
	}

	// This node will die
	if len(n.rules) == 0 {
		return nil
	}

	// Unify all rules
	unifiedRule, err := consolidateRules(n.rules)
	if err != nil {
		return err
	}

	// Rebuild the node based upon the new target version
	if n.targetVersion != unifiedRule.maxVersion {
		//fmt.Printf( "%s: Version change %s -> %s\n", n.Name, n.TargetVersion, unifiedRule.MaxVersion );
		n.validate(unifiedRule.maxVersion)
	}

	return nil
}

/*
 * Given a list of rules, combine them into a single rule, report an
 * error if this cannot be done. The resulting rule has no owner, so this
 * data is lost.
 */
func consolidateRules(rules []rule) (*rule, error) {
	var combine2 func(rule, rule) (*rule, error)
	combine2 = func(r1 rule, r2 rule) (*rule, error) {
		if compareVersionStrings(r1.minVersion, r2.maxVersion) == 1 &&
			compareVersionStrings(r1.maxVersion, r2.maxVersion) == 1 {

			fmt.Println("Unreasolveable 1!")
			return nil, errors.New("Error 1")
		}

		if compareVersionStrings(r1.maxVersion, r2.minVersion) == -1 &&
			compareVersionStrings(r1.maxVersion, r2.maxVersion) == -1 {

			fmt.Println("Unreasolveable 2!")
			return nil, errors.New("Error 2")
		}

		// Sort the four version strings and grab the middle two
		v := [...]string{
			r1.minVersion,
			r1.maxVersion,
			r2.minVersion,
			r2.maxVersion,
		}

		// Shitty sort. It's only four elements, so it probably doesn't
		// matter
		for x := 0; x < 4; x++ {
			for y := x + 1; y < 4; y++ {
				if compareVersionStrings(v[x], v[y]) == 1 {
					tmp := v[x]
					v[x] = v[y]
					v[y] = tmp
				}
			}
		}

		return &rule{
			owner:      nil,
			minVersion: v[1],
			maxVersion: v[2],
		}, nil

	}
	for len(rules) > 1 {
		newRule, err := combine2(rules[0], rules[1])
		if err != nil {
			return nil, err
		}
		rules = rules[2:]
		rules = append(rules, *newRule)
	}

	return &(rules[0]), nil
}

/*
 * Given two version strings, return 1 if the first is greater,
 * 0 if they're equal, and -1 if the second is greater
 * We use the token ^ to represent infinity and _ to represent
 * negative infinity, i.e. no restrictions for max and min, respectively
 */
func compareVersionStrings(str1 string, str2 string) int {
	// I would very much like this gross chain of if statements to not be
	// a thing
	if str1 == "latest" {
		if str2 == "latest" {
			return 0
		}
		return 1
	}
	if str2 == "latest" {
		if str1 == "latest" {
			return 0
		}
		return -1
	}
	if str1 == "_" {
		if str2 == "_" {
			return 0
		}
		return -1
	}
	if str2 == "_" {
		if str1 == "_" {
			return 0
		}
		return 1
	}

	// Tokenize the version strings
	tok1 := tokenize(str1)
	tok2 := tokenize(str2)

	// Compare token arrays
	for index, e := range tok1 {
		if index > len(tok2)-1 {
			return 1
		}
		if e > tok2[index] {
			return 1
		}
		if tok2[index] > e {
			return -1
		}
	}

	if len(tok2) > len(tok1) {
		return -1
	}

	return 0
}
