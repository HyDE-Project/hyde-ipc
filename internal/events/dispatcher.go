package events

import (
	"context"
	"sync"
	"time"

	"github.com/khing/hyde-ipc/internal/executor"
	"github.com/khing/hyde-ipc/internal/utils"
)

type EventDispatcher struct {
	eventQueue  chan EventJob
	workerCount int
	wg          sync.WaitGroup
	ctx         context.Context
	cancelFunc  context.CancelFunc
	executor    *executor.CommandExecutor
}

type EventJob struct {
	EventName string
	EventData string
	Script    string
	Timeout   time.Duration
}

func NewEventDispatcher(workerCount int, queueSize int, cmdExecutor *executor.CommandExecutor) *EventDispatcher {
	ctx, cancel := context.WithCancel(context.Background())

	d := &EventDispatcher{
		eventQueue:  make(chan EventJob, queueSize),
		workerCount: workerCount,
		ctx:         ctx,
		cancelFunc:  cancel,
		executor:    cmdExecutor,
	}

	d.startWorkers()
	return d
}

func (d *EventDispatcher) startWorkers() {
	d.wg.Add(d.workerCount)
	for i := 0; i < d.workerCount; i++ {
		go d.worker(i)
	}
}

func (d *EventDispatcher) worker(id int) {
	defer d.wg.Done()

	utils.LogInfo("Event worker %d started", id)

	for {
		select {
		case <-d.ctx.Done():
			utils.LogInfo("Event worker %d stopping", id)
			return
		case job, ok := <-d.eventQueue:
			if !ok {

				return
			}

			d.processEvent(job, id)
		}
	}
}

func (d *EventDispatcher) processEvent(job EventJob, workerID int) {
	ctx, cancel := context.WithTimeout(d.ctx, job.Timeout)
	defer cancel()

	utils.LogInfo("Worker %d processing event %s", workerID, job.EventName)

	resultCh := make(chan struct {
		output []byte
		err    error
	}, 1)

	go func() {
		output, err := d.executor.Execute(job.Script, job.EventData)
		select {
		case resultCh <- struct {
			output []byte
			err    error
		}{output, err}:
		case <-ctx.Done():
			utils.LogInfo("Script execution for %s was canceled", job.EventName)
		}
	}()

	select {
	case result := <-resultCh:
		if result.err != nil {
			utils.LogInfo("Script error [%s]: %v", job.EventName, result.err)
		} else if utils.Verbose && len(result.output) > 0 {
			utils.LogInfo("Script output [%s]: %s", job.EventName, utils.LimitString(string(result.output), 100))
		}
	case <-ctx.Done():
		utils.LogInfo("Script timeout [%s] after %v", job.EventName, job.Timeout)
	}
}

func (d *EventDispatcher) Dispatch(eventName, eventData, script string, timeout time.Duration) {

	job := EventJob{
		EventName: eventName,
		EventData: eventData,
		Script:    script,
		Timeout:   timeout,
	}

	select {
	case d.eventQueue <- job:

	default:

		utils.LogInfo("Event queue full, dropping event: %s", eventName)
	}
}

func (d *EventDispatcher) Shutdown(timeout time.Duration) {

	d.cancelFunc()

	done := make(chan struct{})
	go func() {
		d.wg.Wait()
		close(done)
	}()

	select {
	case <-done:
		utils.LogInfo("Event dispatcher shut down gracefully")
	case <-time.After(timeout):
		utils.LogInfo("Event dispatcher shutdown timed out")
	}
}
