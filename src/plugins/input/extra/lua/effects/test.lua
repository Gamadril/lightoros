
local width = screen.width
local height = screen.height
local imageData = {}
for x = 1, width do
	imageData[x] = {}
	for y = 1, height do
		imageData[x][y] = { 0, 0, 0 }
	end
end

--Start the write data loop
while not api.isStopRequested() do 
    api.setScreen(imageData)
    
    local idx = 0

    for x=1, width do
        imageData[x][1] = {idx, 0xFF, 1}
        idx = idx + 1
    end

    for y=2, height - 1 do
        imageData[width][y] = {idx, 0, 0xFF}
        idx = idx + 1
    end

    for x=width, 1, -1 do
        imageData[x][height] = {idx, 0x22, 0}
        idx = idx + 1
    end

    for y=height-1, 2, -1 do
        imageData[1][y] = {idx, 0xFF, 0xFF}
        idx = idx + 1
    end

	api.sleep(1000)
end
