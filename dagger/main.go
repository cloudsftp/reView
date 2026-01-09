// A generated module for ReView functions
//
// This module has been generated via dagger init and serves as a reference to
// basic module structure as you get started with Dagger.
//
// Two functions have been pre-created. You can modify, delete, or add to them,
// as needed. They demonstrate usage of arguments and return types using simple
// echo and grep commands. The functions can be called from the dagger CLI or
// from one of the SDKs.
//
// The first line in this comment block is a short description line and the
// rest is a long description with more detail on the module's purpose or usage,
// if appropriate. All modules should have a short description.

package main

import (
	//	"context"
	"dagger/re-view/internal/dagger"
)

const (
	RustVersion = "1.91"

	ToltecImage   = "ghcr.io/toltec-dev/rust"
	ToltecVersion = "v4.0"

	RemarkableTarget = "armv7-unknown-linux-gnueabihf"
)

type ReView struct{}

func (m *ReView) BuildClient(source *dagger.Directory) *dagger.File {
	return linuxContainer(source).
		WithExec([]string{
			"cargo", "build", "--release",
			"--bin", "review-client",
		}).
		WithExec([]string{"cp", "target/release/review-client", "review-client"}).
		File("review-client")
}

func linuxContainer(source *dagger.Directory) *dagger.Container {
	return dag.Container().
		From("rust:"+RustVersion+"-trixie").
		WithExec([]string{"apt", "update"}).
		WithExec([]string{
			"apt", "install", "-y",
			"libgstreamer1.0-dev",
			"libgstreamer-plugins-base1.0-dev",
		}).
		WithDirectory("/source", source).
		WithWorkdir("/source").

		// Cache
		WithMountedCache("/cache/cargo", dag.CacheVolume("rust-packages")).
		WithEnvVariable("CARGO_HOME", "/cache/cargo").
		WithMountedCache("target", dag.CacheVolume("rust-target"))
}

func (m *ReView) BuildServer(source *dagger.Directory) *dagger.File {
	return toltecContainer(source).
		WithExec([]string{
			"cargo", "build", "--release",
			"--bin", "review-server",
			"--target", RemarkableTarget,
		}).
		// WithExec([]string{"cp", "target/" + RemarkableTarget + "/release/review-server", "review-server"}).
		// File("review-server")
		File("target/" + RemarkableTarget + "/release/review-server")
}

func toltecContainer(source *dagger.Directory) *dagger.Container {
	return dag.Container().
		From(ToltecImage+":"+ToltecVersion).
		WithDirectory("/source", source).
		WithWorkdir("/source")

	// Cache
	// WithMountedCache("/cache/cargo", dag.CacheVolume("rust-packages-toltec")).
	// WithEnvVariable("CARGO_HOME", "/cache/cargo").
	// WithMountedCache("target", dag.CacheVolume("rust-target-toltec"))
}
