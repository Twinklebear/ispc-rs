// We just export a simple function which adds two lists of numbers a and b
// and stores the results in c
export void add_lists(const uniform float a[], const uniform float b[],
		uniform float c[], const uniform int count)
{
	// The additions are done in parallel across the program instances
	foreach (i = 0 ... count){
		c[i] = a[i] + b[i];
	}
}

