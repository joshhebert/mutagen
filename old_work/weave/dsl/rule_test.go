package dsl

import (
	"testing"
)

func TestRuleConsolidation(t *testing.T) {
	r := []rule{
		rule{owner: nil, minVersion: "1.0", maxVersion: "2.5"},
		rule{owner: nil, minVersion: "1.4", maxVersion: "2.7"},
		rule{owner: nil, minVersion: "0.9", maxVersion: "1.6"},
	}
	c, err := consolidateRules(r)
	if err != nil {
		t.Fail()
	}
	if c.minVersion != "1.4" {
		t.Fail()
	}
	if c.maxVersion != "1.6" {
		t.Fail()
	}
}
