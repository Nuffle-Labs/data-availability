//go:build darwin

package near

//#cgo LDFLAGS: -L/usr/local/lib -lnear_da_rpc_sys -lm -framework SystemConfiguration -framework Security
import "C"
