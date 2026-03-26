package com.memohai.autofish.data.repository

import com.memohai.autofish.data.model.BindingAddress
import com.memohai.autofish.data.model.AppLanguage
import com.memohai.autofish.data.model.AppThemeMode
import com.memohai.autofish.data.model.ServerConfig
import kotlinx.coroutines.flow.Flow

interface SettingsRepository {
    val serverConfig: Flow<ServerConfig>

    suspend fun getServerConfig(): ServerConfig
    suspend fun updatePort(port: Int)
    suspend fun updateBindingAddress(bindingAddress: BindingAddress)
    suspend fun updateBearerToken(token: String)
    suspend fun generateNewBearerToken(): String
    suspend fun updateAutoStartOnBoot(enabled: Boolean)
    suspend fun updateRestPort(port: Int)
    suspend fun updateRestBearerToken(token: String)
    suspend fun generateNewRestBearerToken(): String
    suspend fun updateRestOverlayVisible(visible: Boolean)
    suspend fun updateAppLanguage(language: AppLanguage)
    suspend fun updateAppThemeMode(themeMode: AppThemeMode)

    fun validatePort(port: Int): Result<Int>
}
