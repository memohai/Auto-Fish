package com.memohai.autofish.mcp.tools

import com.memohai.autofish.services.system.ToolRouter
import io.modelcontextprotocol.kotlin.sdk.server.Server
import io.modelcontextprotocol.kotlin.sdk.types.ToolSchema
import kotlinx.serialization.json.buildJsonObject

object SystemActionTools {
    fun register(server: Server, toolRouter: ToolRouter) {
        val emptySchema = ToolSchema(properties = buildJsonObject {})

        server.addTool(name = "autofish_press_back", description = "Press the Back button", inputSchema = emptySchema) {
            toolRouter.pressBack().toCallToolResult("Pressed Back")
        }

        server.addTool(name = "autofish_press_home", description = "Press the Home button", inputSchema = emptySchema) {
            toolRouter.pressHome().toCallToolResult("Pressed Home")
        }
    }
}
