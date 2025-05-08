package executor

import (
	"context"
	"fmt"
	"os"
	"os/exec"
	"sync"
	"time"
)

type CommandExecutor struct {
	limit    chan struct{}
	execLock sync.Mutex
	killMap  sync.Map
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

	cmdID := fmt.Sprintf("%p", cmd)
	e.killMap.Store(cmdID, cmd)
	defer e.killMap.Delete(cmdID)

	result, err := cmd.CombinedOutput()
	return result, err
}

func (e *CommandExecutor) ExecuteWithTimeout(script, eventData string, timeout time.Duration) ([]byte, error) {

	ctx, cancel := context.WithTimeout(context.Background(), timeout)
	defer cancel()

	resultCh := make(chan struct {
		output []byte
		err    error
	}, 1)

	go func() {
		output, err := e.Execute(script, eventData)
		select {
		case resultCh <- struct {
			output []byte
			err    error
		}{output, err}:
		case <-ctx.Done():

		}
	}()

	select {
	case result := <-resultCh:
		return result.output, result.err
	case <-ctx.Done():
		return nil, fmt.Errorf("command timed out after %v", timeout)
	}
}

func (e *CommandExecutor) KillAllCommands() {
	e.killMap.Range(func(key, value interface{}) bool {
		if cmd, ok := value.(*exec.Cmd); ok && cmd.Process != nil {
			cmd.Process.Kill()
		}
		return true
	})
}
