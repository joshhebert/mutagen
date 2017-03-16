package dsl

import (
	"testing"
)

func TestUnmarshal(t *testing.T) {
	/*
	json := "{\"Target\" : { \"Name\" : \"vim\", \"Version\" : \"4.5\" }, \"Require\" : [ { \"Name\" : \"library1\", \"MinVersion\" : \"_\", \"MaxVersion\" : \"1.5\" }, { \"Name\" : \"library2\", \"MinVersion\" : \"2.0\", \"MaxVersion\" : \"^\" } ]}"

	var mf manifest
	mf.unmarshall(json)
	if mf.target.name != "vim" {
		fmt.Printf("Expect %s as the package name, but got %s\n", "vim", mf.target.name)
		t.Fail()
	}
	*/
}

func TestTranspiler(t *testing.T) {
	//mf := transpileManifest( "[PACKAGE]\n\tvim : 4.5\n[REQUIRED]\n\tlibrary1 : _1.5\n\tlibrary2 : ^2.0" )
	//mf := transpile_manifest_json( "[REQUIRED]\n\tlibrary1 : _1.5\n\tlibrary2 : ^2.0" )

	//fmt.Println( mf )
	/*
	   if( mf.Target.Name != "vim" ){
	       t.Fail( )
	   }

	   if( mf.Require[ 0 ].Name != "library1" ){
	       t.Fail( )
	   }
	   if( mf.Require[ 0 ].MinVersion != "_" ){
	       t.Fail( )
	   }
	   if( mf.Require[ 0 ].MaxVersion != "1.5" ){
	       t.Fail( )
	   }


	   if( mf.Require[ 1 ].Name != "library2" ){
	       t.Fail( )
	   }
	   if( mf.Require[ 1 ].MinVersion != "2.0" ){
	       t.Fail( )
	   }
	   if( mf.Require[ 1 ].MaxVersion != "^" ){
	       t.Fail( )
	   }
	*/
}

/*
 * Finally, test the comparator we use to organize everything, just in case
 */
func TestCompareVersionStrings(t *testing.T) {
	if compareVersionStrings("1.2.3.4", "1.2.3.5") != -1 {
		t.Fail()
	}

	if compareVersionStrings("1.2.3-4", "1.2.3-5") != -1 {
		t.Fail()
	}

	if compareVersionStrings("1.2.3.4", "1.3") != -1 {
		t.Fail()
	}

	if compareVersionStrings("1.2", "1.3.4.5") != -1 {
		t.Fail()
	}

	if compareVersionStrings("1.3", "1.2.3.4") != 1 {
		t.Fail()
	}

	if compareVersionStrings("1.2.3.5", "1.2.3.4") != 1 {
		t.Fail()
	}

	if compareVersionStrings("1.3.4.5", "1.2.3.5") != 1 {
		t.Fail()
	}
}
