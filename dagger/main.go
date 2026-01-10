package main

import (
	"dagger/re-view/internal/dagger"
)

const (
	RustVersion = "1.92"

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

		// Sources
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
			"bash", "-c",
			"source /opt/x-tools/switch-arm.sh; " +
				"cargo build --release --bin review-server --target " + RemarkableTarget,
		}).
		WithExec(
			[]string{"cp", "target/" + RemarkableTarget + "/release/review-server", "review-server"},
		).
		File("review-server")
}

func toltecContainer(source *dagger.Directory) *dagger.Container {
	return dag.Container().
		From(ToltecImage+":"+ToltecVersion).

		// Sources
		WithDirectory("/source", source).
		WithWorkdir("/source")

	// Sadly caching breaks compile :(
}
