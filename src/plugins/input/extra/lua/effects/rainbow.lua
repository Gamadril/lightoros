
local width = screen.width
local height = screen.height

local imageData = {} -- matrix: width * height of (0,0,0)
for x = 1, width do
	imageData[x] = {}
	for y = 1, height do
		imageData[x][y] = {0, 0, 0}
	end
end

local hue = 0
local step = 360 / (2 * width + 2 * (height - 2)) 

while not api.isStopRequested() do
	api.setScreen(imageData)
	local h = hue

	for i = 1,width do
		imageData[i][1] = { color.hsv2rgb(h,255,255) }
		h = (h + step) % 360
	end

	for i = 2,height-1 do
		imageData[width][i] = { color.hsv2rgb(h,255,255) }
		h = (h + step) % 360
	end

	for i = width,1,-1 do
		imageData[i][height] = { color.hsv2rgb(h,255,255) }
		h = (h + step) % 360
	end

	for i = height-1,2,-1 do
		imageData[1][i] = { color.hsv2rgb(h,255,255) }
		h = (h + step) % 360
	end

	api.sleep(100)
	hue = (hue + 5) % 360
end