package ibu

import (
	"fmt"
	"hash/fnv"
	"os"
	"syscall"
)

// TODO
// We need to test if the overlay actually is mounted. Because if it's not,
// unmounting the root FS is _really_ bad
// Very likely keep this lock in /tmp and check for its existance
func lazyUnmount(target string) {
	// Unmount the overlayfs mounted at target lazily, i.e. ignore that things
	// are almost certainly using it.

	// For now though, wing it
	syscall.Unmount(target, syscall.MNT_DETACH)
}

// Overlay the given directory onto another one
func overlay(top string, onto string, work_dir string) error {
	var mount_flags = syscall.MS_NOEXEC | syscall.MS_NOSUID | syscall.MS_NODEV
	hash := fnv.New64a()
	hash.Write([]byte(top + onto))
	// TODO We need to guarantee that the work dir is on the same partition
	// as the upper dir
	// TODO Are these the right permissions?
	os.MkdirAll(work_dir, 0777)
	fmt.Println(work_dir)

	var options = fmt.Sprintf("upperdir=%s,lowerdir=%s,workdir=%s", top, onto, work_dir)

	syscall.Mount(
		"overlay",
		onto,
		"overlay",
		uintptr(mount_flags),
		options)

	return nil
}

// Given a filesystem, overlay it onto root on a directory-by-directory basis
func hotswap(source string) {
	fmt.Printf("Hold onto your butts...\n")

	// Our work dirs double as our cache. We can read /proc/mounts and look for
	// mount entries that use these work dirs. If we find one that uses that
	// work dir, we need to unmount it, VERIFY THAT IT'S UNMOUNTED, and clear
	// the work dir from the FS.
	//lazyUnmount("/")

	//for each dir d in source{
	//	overlay(d, "/" + d, "some_work_dir" /* TODO */)
	//}

	//var work_dir = top + "/tmp/.weavelock/" + strconv.FormatUint(hash.Sum64(), 10)
}
