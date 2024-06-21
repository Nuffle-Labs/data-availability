package plasma_test

import (
	"os"
	"testing"

	plasma "github.com/nuffle-labs/data-availability/test/plasma"
)

func TestMain(m *testing.M) {
	plasma.ParseEnv()
	code := m.Run()
	os.Exit(code)
}
