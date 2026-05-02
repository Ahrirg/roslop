local ServerScriptService = game:GetService("ServerScriptService")
local HttpService = game:GetService("HttpService")
local Selection = game:GetService("Selection")
local toolbar = plugin:CreateToolbar("Custom")

-- Add a toolbar button labeled "Empty Script"
local newScriptButton = toolbar:CreateButton("Empty Script", "Create an empty script", "rbxassetid://14978048121")



local function getFileTree()
	local VALID_SOURCES = { game.Workspace, game.ServerScriptService, game.StarterPlayer.StarterPlayerScripts }
	
	
	local function recursion(curInstance: Instance, depth: number)
		local result = ""

		if curInstance:IsA("Script") or curInstance:IsA("LocalScript") or curInstance:IsA("ModuleScript") or curInstance:FindFirstChildWhichIsA("Script")  then
			result ..= string.rep("  ", depth) .. curInstance.Name .." <|NL|>"
		end
		

		for _, v in ipairs(curInstance:GetChildren()) do
			result ..= recursion(v, depth + 1)
		end

		return result
	end
	
	local str = "<|NL|>"
	for _, v in ipairs(VALID_SOURCES) do
		str ..= "<|NL|>"
		str ..= recursion(v, 0)
	end

	return str
end

local function getScriptContents(filepath: string)
	local path = string.split(filepath, "/")
	local curInstance = game
	for _, v in ipairs(path) do
		curInstance = curInstance:FindFirstChild(v)
		if not curInstance then
			return "File was not found"
		end
	end
	
	return string.gsub((curInstance :: Script).Source, "\n", "<|NL|>")
end


local function overrideScriptContents(filepath: string, content: string)
	local path = string.split(filepath, "/")
	local curInstance = game
	for _, v in ipairs(path) do
		curInstance = curInstance:FindFirstChild(v)
		if not curInstance then
			return "File was not found"
		end
	end
	
	local contents = string.gsub(content, "<|NL|>", "\n")
	print(contents);
	(curInstance :: Script).Source = contents
	return "Script has been successfully overriden!"
end

local function connect()
	local ws_client = HttpService:CreateWebStreamClient(Enum.WebStreamClientType.WebSocket, {
		Url = "ws://localhost:6967/ws"
	})
	local function sendMessage(msg: string, id: string)
		local response = {
			response = msg,
			id = id,
			error = "None"
		}

		ws_client:Send(HttpService:JSONEncode(response))
	end
	
	local function handleMessage(message)
		local data = HttpService:JSONDecode(message)
		local function respondMessage(msg) sendMessage(msg, data.id) end

		print("Server said:")
		print(data)

		if data.command == "FileTree" then
			respondMessage(getFileTree())
		elseif string.sub(data.command, 1, 7) == "Content" then
			local args = string.split(data.command, " ")
			respondMessage(getScriptContents(args[2]))
		elseif string.sub(data.command, 1, 8) == "Override" then
			data.command = string.gsub(data.command, "lua", "")
			data.command = string.gsub(data.command, "```", "")
			
			local args = string.split(data.command, " ")
			print(args)
			local script_text = string.sub(data.command, #args[1] + #args[2] + 2, #data.command)
			print(args[2])
			print(script_text)
			respondMessage(overrideScriptContents(args[2], script_text))
		else
			respondMessage("Command not found")
		end
	end

	ws_client.MessageReceived:Connect(handleMessage)
	
	local msg = {
		response = "ping",
		error = "None",
	}
	ws_client:Send(HttpService:JSONEncode(msg))

	return ws_client
end


newScriptButton.ClickableWhenViewportHidden = true

local function onPluginButtonClicked()
	connect()
end

newScriptButton.Click:Connect(onPluginButtonClicked)