# Introduction
A Factorio calculator project written in rust, built with mod support in mind.

# Usage
The program loads everything from a data dump, resulting in almost perfect mod support.
To create a data dump, run
`path-to-executable/factorio.exe --dump-data`. The resulting data dump will be saved to your "script-output" folder.
You will need a data dump to create a project.

Use **Arrow keys** or **WASD** to move around. Select menus with **Space** or **Enter**, and return using **Esc**. **F** can be used to search in lists.

Add **Inputs**, **Outputs** and **Processes** for the model to solve.
In the **Inputs** and **Outputs** menu, you can set the amount (maximum for inputs, minimum for outputs). 
From the **Processes** menu, you can select the **Machine** doing the process.
after selecting a **Machine**, you can modify **Modules** and **Beacons**. 

**Solve** the model and see the results:
* **No solution** means there's no feasible solution for your model. (proper hints will be added in the future.)
* **Unbounded** means there outputs can be increased indefinitely. Perhaps you have no inputs and only use resources? Try using the generative solve.
* **Multiple solutions** means there are more than one solution to your model. Follow the hints and tighten your limits to reach a single solution.
* **One solution** shows you the solution for your model: how many machines you need for each process.

**Solve and generate inputs** will automatically generate inputs for your model. Useful if you don't want to add the inputs manually. Also works if there are no inputs to your model. This mode does NOT try to maximize the outputs. 