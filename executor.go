package main

import (
	"os"
	"os/exec"
	"sync"
	"time"
)

type CommandExecutor struct {
	limit    chan struct{}
	execLock sync.Mutex
}

func NewCommandExecutor(maxConcurrent int) *CommandExecutor {
	if maxConcurrent <= 0 {
		maxConcurrent = 4
	}
	return &CommandExecutor{
		limit: make(chan struct{}, maxConcurrent),
	}
}

func (e *CommandExecutor) Execute(script, eventData string) ([]byte, error) {

	e.limit <- struct{}{}
	defer func() { <-e.limit }()

	cmd := exec.Command("sh", "-c", script)
	cmd.Env = append(os.Environ(), "HYDE_EVENT_DATA="+eventData)

	e.execLock.Lock()
	result, err := cmd.CombinedOutput()
	e.execLock.Unlock()

	return result, err
}

func (e *CommandExecutor) ExecuteWithTimeout(script, eventData string, timeout time.Duration) ([]byte, error) {

	done := make(chan struct{})
	var result []byte
	var err error

	go func() {
		result, err = e.Execute(script, eventData)
		close(done)
	}()

	select {
	case <-done:
		return result, err
	case <-time.After(timeout):
		return nil, errTimeout
	}
}

var errTimeout = exec.ErrNotFound
