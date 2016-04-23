task void hello_task(){
	print("hello_task: programIndex: %\n", programIndex);
	print("hello_task: task indices [%, %, %]\n", taskIndex0, taskIndex1, taskIndex2);
}

export void run_tasks(){
	launch hello_task();
}

