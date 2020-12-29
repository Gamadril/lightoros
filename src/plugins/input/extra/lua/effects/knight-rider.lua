local speed = args.speed or 1.0
local fadeFactor = args.fadeFactor or 0.7
local color = args.color or {255,0,0}

speed = math.max(0.0001, speed)
fadeFactor = math.max(0.0, math.min(fadeFactor, 1.0))

-- Initialize the led data
local width = screen.width
local height = screen.height
local imageData = {}
for x = 1, width do
	imageData[x] = {}
	for y = 1, height do
		imageData[x][y] = { 0, 0, 0 }
	end
end

-- Calculate the sleep time and rotation increment
local increment = 1
local sleepTime = 1000 / (speed * width)
while sleepTime < 50 do
	increment = increment * 2
	sleepTime = sleepTime * 2
end

--Start the write data loop
local position = 0
local direction = 1
while not api.isStopRequested() do 
	api.setScreen(imageData)

	-- Move data into next state
	for i=1, increment do
		position = position + direction 
		if position == 0 then
			position = 1
			direction = 1
		elseif position == width+1 then 
			position = width
			direction = -1
		end

		-- Fade the old data
		for j=1, width do
			for y = 1, height do
				imageData[j][y] = {
					math.floor(fadeFactor * imageData[j][y][1]),
					math.floor(fadeFactor * imageData[j][y][2]),
					math.floor(fadeFactor * imageData[j][y][3])
				}
			end
		end

		-- Insert new data
		for y = 1, height do
			imageData[position][y] = color
		end
	end

	-- Sleep for a while
	api.sleep(sleepTime)
end
