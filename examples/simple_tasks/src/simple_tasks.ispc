task void hello_task(){
	print("Hello!, programIndex: %", programIndex);
}

export void run_tasks(){
	launch hello_task();
}

