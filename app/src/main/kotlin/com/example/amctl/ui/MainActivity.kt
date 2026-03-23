package com.example.amctl.ui

import android.os.Bundle
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.viewModels
import androidx.appcompat.app.AppCompatActivity
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.appcompat.app.AppCompatDelegate
import androidx.core.os.LocaleListCompat
import androidx.lifecycle.lifecycleScope
import com.example.amctl.data.model.AppLanguage
import com.example.amctl.data.model.AppThemeMode
import com.example.amctl.data.repository.SettingsRepository
import com.example.amctl.ui.screens.HomeScreen
import com.example.amctl.ui.theme.AmctlTheme
import com.example.amctl.ui.viewmodels.MainViewModel
import dagger.hilt.android.AndroidEntryPoint
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import javax.inject.Inject

@AndroidEntryPoint
class MainActivity : AppCompatActivity() {
    @Inject lateinit var settingsRepository: SettingsRepository
    private val viewModel: MainViewModel by viewModels()

    override fun onCreate(savedInstanceState: Bundle?) {
        val initialThemeMode = runBlocking {
            settingsRepository.serverConfig.first().appThemeMode
        }
        AppCompatDelegate.setDefaultNightMode(
            if (initialThemeMode == AppThemeMode.DARK) {
                AppCompatDelegate.MODE_NIGHT_YES
            } else {
                AppCompatDelegate.MODE_NIGHT_NO
            },
        )
        super.onCreate(savedInstanceState)

        enableEdgeToEdge()
        setContent {
            val appThemeMode by viewModel.serverConfig
                .map { it.appThemeMode }
                .collectAsState(initial = initialThemeMode)

            AmctlTheme(
                darkTheme = appThemeMode == AppThemeMode.DARK,
                dynamicColor = false,
            ) {
                HomeScreen()
            }
        }

        lifecycleScope.launch {
            val config = settingsRepository.getServerConfig()
            val localeTags = when (config.appLanguage) {
                AppLanguage.SYSTEM -> ""
                AppLanguage.CHINESE -> "zh"
                AppLanguage.ENGLISH -> "en"
            }
            val currentTags = AppCompatDelegate.getApplicationLocales().toLanguageTags()
            if (currentTags != localeTags) {
                AppCompatDelegate.setApplicationLocales(LocaleListCompat.forLanguageTags(localeTags))
            }
        }
    }
}
