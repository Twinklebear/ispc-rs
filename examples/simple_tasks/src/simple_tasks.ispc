task void very_silly_task(){
	print("very_silly_task programIndex: %, threadIndex %\n", programIndex, threadIndex);
}
task void silly_task(int launch_num){
	print("silly_task %: programIndex: %, threadIndex %\n", launch_num, programIndex, threadIndex);
	launch[2] very_silly_task();
	launch[2] very_silly_task();
}
task void hello_task(){
	print("hello_task: programIndex: %, threadIndex %\n", programIndex, threadIndex);
	print("hello_task: task indices [%, %, %]\n", taskIndex0, taskIndex1, taskIndex2);
	launch[8] silly_task(0);
	launch[8] silly_task(0);
}
task void goodbye_task(){
	print("goodbye_task: programIndex: %, threadIndex %\n", programIndex, threadIndex);
	print("goodbye_task: task indices [%, %, %]\n", taskIndex0, taskIndex1, taskIndex2);
}

export void run_tasks(){
	// The first launch of a function calls ISPCAlloc with a null handle_ptr
	// creating a new task context
	launch[16] hello_task();
	launch[16] hello_task();
	launch[16] hello_task();
	launch[16] hello_task();
	// Sync'ing will wait for all existing tasks in the context to finish
	sync;
	// The next launch after an explicit sync will pass a null handle_ptr again
	// since it's a new context again
	//launch[4] hello_task();
	launch[2, 2, 2] goodbye_task();
}

