package utils

import (
	"log"
)

var Verbose bool

func LogInfo(format string, v ...interface{}) {
	if Verbose {
		log.Printf(format, v...)
	}
}

func LimitString(s string, maxLen int) string {
	if len(s) <= maxLen {
		return s
	}
	return s[:maxLen] + "..."
}

func SetupLogging(verbose bool) {
	Verbose = verbose

	log.SetPrefix("hyde-ipc: ")
	if verbose {
		log.SetFlags(log.Ldate | log.Ltime)
	} else {
		log.SetFlags(0)
	}
}
