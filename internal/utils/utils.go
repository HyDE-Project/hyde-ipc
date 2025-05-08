package utils

import (
	"log"
)

// Global variables for logging
var Verbose bool

// LogInfo logs an info message if verbose mode is enabled
func LogInfo(format string, v ...interface{}) {
	if Verbose {
		log.Printf(format, v...)
	}
}

// LimitString truncates a string to the specified maximum length
func LimitString(s string, maxLen int) string {
	if len(s) <= maxLen {
		return s
	}
	return s[:maxLen] + "..."
}

// SetupLogging configures the logger with the specified verbosity level
func SetupLogging(verbose bool) {
	Verbose = verbose

	log.SetPrefix("hyde-ipc: ")
	if verbose {
		log.SetFlags(log.Ldate | log.Ltime)
	} else {
		log.SetFlags(0)
	}
}
