task void leaf_task(int x){
	print("Leaf task %\n", x);
}
task void sample_task(uniform int val){
	foreach (i = 0 ... val){
		print("Task % running, program index %\n", taskIndex0, programIndex);
	}
	launch[2] leaf_task(taskIndex0);
}
export void custom_tasksys(uniform int num_tasks){
	launch[num_tasks] sample_task(programCount);
	uniform int half_tasks = num_tasks;
	if (half_tasks == 0){
		half_tasks = 1;
	}
	launch[half_tasks] sample_task(programCount * 2);
}

