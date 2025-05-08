package executor

import (
	"context"
	"fmt"
	"os"
	"os/exec"
	"sync"
	"time"
)

// CommandExecutor manages script execution with minimal memory overhead
type CommandExecutor struct {
	limit    chan struct{} // Semaphore to limit concurrent executions
	execLock sync.Mutex    // Lock for command execution
	killMap  sync.Map      // Map to track running commands for potential termination
}

// NewCommandExecutor creates a new executor with concurrency limiting
func NewCommandExecutor(maxConcurrent int) *CommandExecutor {
	if maxConcurrent <= 0 {
		maxConcurrent = 4 // Reasonable default
	}
	return &CommandExecutor{
		limit: make(chan struct{}, maxConcurrent),
	}
}

// Execute runs a shell command with environment variables
func (e *CommandExecutor) Execute(script, eventData string) ([]byte, error) {
	// Acquire semaphore slot or wait
	e.limit <- struct{}{}
	defer func() { <-e.limit }()

	// Prepare command with minimal allocations
	cmd := exec.Command("sh", "-c", script)
	cmd.Env = append(os.Environ(), "HYDE_EVENT_DATA="+eventData)

	// Store command in the kill map with a unique ID
	cmdID := fmt.Sprintf("%p", cmd)
	e.killMap.Store(cmdID, cmd)
	defer e.killMap.Delete(cmdID)

	// Run command - no lock needed as each runs in its own goroutine
	result, err := cmd.CombinedOutput()
	return result, err
}

// ExecuteWithTimeout runs a command with a timeout
func (e *CommandExecutor) ExecuteWithTimeout(script, eventData string, timeout time.Duration) ([]byte, error) {
	// Create a context with timeout
	ctx, cancel := context.WithTimeout(context.Background(), timeout)
	defer cancel()

	// Create channel for results
	resultCh := make(chan struct {
		output []byte
		err    error
	}, 1)

	// Run the command in a goroutine
	go func() {
		output, err := e.Execute(script, eventData)
		select {
		case resultCh <- struct {
			output []byte
			err    error
		}{output, err}:
		case <-ctx.Done():
			// Context is done, don't send results
		}
	}()

	// Wait for result or timeout
	select {
	case result := <-resultCh:
		return result.output, result.err
	case <-ctx.Done():
		return nil, fmt.Errorf("command timed out after %v", timeout)
	}
}

// KillAllCommands forcefully terminates all running commands
func (e *CommandExecutor) KillAllCommands() {
	e.killMap.Range(func(key, value interface{}) bool {
		if cmd, ok := value.(*exec.Cmd); ok && cmd.Process != nil {
			cmd.Process.Kill()
		}
		return true // continue iteration
	})
}
