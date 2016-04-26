task void hello_task(){
	print("hello_task: programIndex: %\n", programIndex);
	print("hello_task: task indices [%, %, %]\n", taskIndex0, taskIndex1, taskIndex2);
}

task void goodbye_task(){
	print("goodbye_task: programIndex: %\n", programIndex);
	print("goodbye_task: task indices [%, %, %]\n", taskIndex0, taskIndex1, taskIndex2);
}

export void run_tasks(){
	// The first launch of a function calls ISPCAlloc with a null handle_ptr
	// creating a new task context
	launch hello_task();
	// Sync'ing will wait for all existing tasks in the context to finish
	sync;
	// The next launch after an explicit sync will pass a null handle_ptr again
	// since it's a new context again
	launch hello_task();
	launch goodbye_task();
}

