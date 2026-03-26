package com.memohai.autofish.mcp

import com.memohai.autofish.mcp.auth.BearerTokenAuth
import com.memohai.autofish.mcp.tools.AppTools
import com.memohai.autofish.mcp.tools.NodeActionTools
import com.memohai.autofish.mcp.tools.ScreenIntrospectionTools
import com.memohai.autofish.mcp.tools.SystemActionTools
import com.memohai.autofish.mcp.tools.TextInputTools
import com.memohai.autofish.mcp.tools.TouchActionTools
import com.memohai.autofish.mcp.tools.UtilityTools
import com.memohai.autofish.services.accessibility.AccessibilityServiceProvider
import com.memohai.autofish.services.accessibility.AccessibilityTreeParser
import com.memohai.autofish.services.accessibility.ActionExecutor
import com.memohai.autofish.services.accessibility.CompactTreeFormatter
import com.memohai.autofish.services.accessibility.ElementFinder
import com.memohai.autofish.services.system.ToolRouter
import io.ktor.http.ContentType
import io.ktor.http.HttpStatusCode
import io.ktor.serialization.kotlinx.json.json
import io.ktor.server.application.install
import io.ktor.server.engine.EmbeddedServer
import io.ktor.server.engine.embeddedServer
import io.ktor.server.netty.Netty
import io.ktor.server.netty.NettyApplicationEngine
import io.ktor.server.plugins.contentnegotiation.ContentNegotiation
import io.ktor.server.response.respondText
import io.ktor.server.routing.get
import io.ktor.server.routing.routing
import io.modelcontextprotocol.kotlin.sdk.server.Server
import io.modelcontextprotocol.kotlin.sdk.server.ServerOptions
import io.modelcontextprotocol.kotlin.sdk.types.Implementation
import io.modelcontextprotocol.kotlin.sdk.types.McpJson
import io.modelcontextprotocol.kotlin.sdk.types.ServerCapabilities

class McpServer(
    private val port: Int,
    private val bindAddress: String,
    private val bearerToken: String,
    private val accessibilityServiceProvider: AccessibilityServiceProvider,
    private val treeParser: AccessibilityTreeParser,
    private val compactTreeFormatter: CompactTreeFormatter,
    private val elementFinder: ElementFinder,
    private val actionExecutor: ActionExecutor,
    private val toolRouter: ToolRouter,
) {
    private var server: EmbeddedServer<NettyApplicationEngine, NettyApplicationEngine.Configuration>? = null

    fun start() {
        val mcpSdkServer = Server(
            Implementation(name = "auto-fish", version = "0.2.0"),
            ServerOptions(capabilities = ServerCapabilities(tools = ServerCapabilities.Tools())),
        )

        ScreenIntrospectionTools.register(mcpSdkServer, accessibilityServiceProvider, treeParser, compactTreeFormatter, toolRouter)
        TouchActionTools.register(mcpSdkServer, toolRouter)
        NodeActionTools.register(mcpSdkServer, actionExecutor, elementFinder, accessibilityServiceProvider, treeParser)
        TextInputTools.register(mcpSdkServer, actionExecutor, accessibilityServiceProvider, treeParser, toolRouter)
        SystemActionTools.register(mcpSdkServer, toolRouter)
        UtilityTools.register(mcpSdkServer, elementFinder, accessibilityServiceProvider, treeParser)
        AppTools.register(mcpSdkServer, toolRouter)

        server = embeddedServer(
            factory = Netty,
            port = port,
            host = bindAddress,
        ) {
            install(ContentNegotiation) { json(McpJson) }
            install(BearerTokenAuth) { token = bearerToken }

            routing {
                get("/health") {
                    call.respondText("""{"status":"healthy"}""", ContentType.Application.Json, HttpStatusCode.OK)
                }
            }

            mcpStreamableHttp { mcpSdkServer }
        }.start(wait = false)
    }

    fun stop() {
        server?.stop(gracePeriodMillis = 1000, timeoutMillis = 5000)
        server = null
    }
}
