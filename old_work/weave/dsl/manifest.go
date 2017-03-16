package dsl

import (
	"encoding/json"
	"errors"
	"fmt"
	"io/ioutil"
	"regexp"
	"strconv"
	"strings"
)

type concretePackage struct {
	name    string
	version string
}

type loosePackage struct {
	name       string
	minVersion string
	maxVersion string
}

type manifest struct {
	target  concretePackage
	require []loosePackage
}

func (mf MultiManifestResolver) getManifest(name string, version string) (*manifest, error) {
	for _, e := range mf.Resolvers {
		m, err := e.getManifest(name, version)
		if err != nil {
			return &(manifest{}), err
		}

		if m != nil {
			return m, nil
		}
	}

	return &(manifest{}), errors.New("Could not resolve manifest")
}

func (mf ManifestNetResolver) getManifest(name string, version string) (*manifest, error) {
	fmt.Println("Not implemented!")
	return &(manifest{}), errors.New("Not implemented")
}

func (m ManifestFSResolver) getManifest(name string, version string) (*manifest, error) {
	// Slurp the manifest. They aren't that big, so this shouldn't cause
	// problems
	if version == "^" {
		version = "latest"
	}
	manifestPath := name + "-" + version + ".weave"
	data, err := ioutil.ReadFile("./mf/" + manifestPath)
	if err != nil {
		//fmt.Printf( "Error opening '%s'\n", manifestPath )
		//fmt.Println( err )
		return &(manifest{}), err
	}
	var mf *manifest
	mf.unmarshall(string(data))
	return mf, nil
}

func tokenize(s string) []int {
	strToks := strings.FieldsFunc(s, func(r rune) bool {
		switch r {
		case '.', '-':
			return true
		}
		return false
	})

	var ret []int
	for i := 0; i < len(strToks); i++ {
		t, err := strconv.Atoi(strToks[i])
		if err != nil {
			fmt.Println(err)
		}
		ret = append(ret, t)
	}

	return ret
}

/*
 * Due to how Go unmarshalls JSON, the format it needs
 * to be in is far from user friendly. So, in order users
 * (and dev testers!) to write manifests more easily, we convert
 * from an sugared JSON string to something that Go can unmarshall
 * into a manifest struct
 * Our sugar is as follows:
 *      1.0-2.0 -> Anything between 1.0 and 2.0, inclusive
 *      ^1.0    -> Anything greater than 1.0
 *      _2.0    -> 2.0 and below, including 2.0
 *      *       -> Anything (no version restrictions)
 *
 * I'll probably add better operator control in time, but these
 * will do for now
 */
func transpileManifest(in string) string {
	var name string
	var version string
	var depList []loosePackage
	lines := strings.Split(in, "\n")
	for len(lines) > 0 {
		switch lines[0] {
		case "[PACKAGE]":
			lines = lines[1:]
			re := regexp.MustCompile("^\t([a-zA-Z]|-|[0-9])+ : ([0-9]+(\\.|-))*[0-9]+$")
			if re.MatchString(lines[0]) {
				// Extract out name and version
				fmt.Sscanf(lines[0], "\t%s : %s", &name, &version)
			} else {
				// Syntax Error
			}
			lines = lines[1:]
			break
		case "[REQUIRED]":
			for len(lines) > 1 {
				lines = lines[1:]
				// Mmm... regex. Match only things in the format of dependencies
				re := regexp.MustCompile("^\t([a-zA-Z]|-|[0-9])+ : (\\*|(\\^|_)([0-9]+(\\.|-))*[0-9]+)$")
				if re.MatchString(lines[0]) {
					var d loosePackage
					var v string
					fmt.Sscanf(lines[0], "\t%s : %s", &(d.name), &v)
					// We need to convert v_string to a min and max version
					// TODO
					depList = append(depList, loosePackage{
						name:       "fill me",
						minVersion: "fill me",
						maxVersion: "fill me",
					})
				} else {
					break
				}
			}
			lines = lines[1:]
			break
		default:
			fmt.Println("Syntax Error")
			fmt.Println(lines[0])
			lines = lines[1:]
		}
	}

	return fmt.Sprintf("{\"Target\":{\"Name\":\"%s\",\"Version\":\"%s\"},\"Require\":null}", name, version)
}

func (mf *manifest) unmarshall(in string) {

	var JSONBlob = []byte(in)

	// Unmarshall the manifest (byte blob) into a manifest (struct)
	err2 := json.Unmarshal(JSONBlob, mf)
	if err2 != nil {
		fmt.Println("JSON error:", err2)
	}
}
