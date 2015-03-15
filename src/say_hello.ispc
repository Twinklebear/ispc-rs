
export void say_hello(){
	foreach (i = 0 ... 16) {
		print("Hello from ISPC program %, index %!\n", programIndex, i);
	}
}

