package com.memohai.autofish.di

import android.content.Context
import androidx.datastore.core.DataStore
import androidx.datastore.preferences.core.Preferences
import androidx.datastore.preferences.preferencesDataStore
import com.memohai.autofish.data.repository.SettingsRepository
import com.memohai.autofish.data.repository.SettingsRepositoryImpl
import com.memohai.autofish.services.accessibility.AccessibilityServiceProvider
import com.memohai.autofish.services.accessibility.AccessibilityServiceProviderImpl
import com.memohai.autofish.services.accessibility.ActionExecutor
import com.memohai.autofish.services.accessibility.ActionExecutorImpl
import com.memohai.autofish.services.screencapture.ScreenCaptureProvider
import com.memohai.autofish.services.screencapture.ScreenCaptureProviderImpl
import com.memohai.autofish.services.system.AppController
import com.memohai.autofish.services.system.AppControllerImpl
import com.memohai.autofish.services.system.ShizukuProvider
import com.memohai.autofish.services.system.ShizukuProviderImpl
import dagger.Binds
import dagger.Module
import dagger.Provides
import dagger.hilt.InstallIn
import dagger.hilt.android.qualifiers.ApplicationContext
import dagger.hilt.components.SingletonComponent
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.Dispatchers
import javax.inject.Qualifier
import javax.inject.Singleton

@Qualifier
@Retention(AnnotationRetention.BINARY)
annotation class IoDispatcher

private val Context.settingsDataStore: DataStore<Preferences> by preferencesDataStore(
    name = "settings",
)

@Module
@InstallIn(SingletonComponent::class)
object AppModule {
    @Provides
    @Singleton
    fun provideDataStore(
        @ApplicationContext context: Context,
    ): DataStore<Preferences> = context.settingsDataStore

    @Provides
    @IoDispatcher
    fun provideIoDispatcher(): CoroutineDispatcher = Dispatchers.IO
}

@Module
@InstallIn(SingletonComponent::class)
abstract class RepositoryModule {
    @Binds
    @Singleton
    abstract fun bindSettingsRepository(impl: SettingsRepositoryImpl): SettingsRepository
}

@Module
@InstallIn(SingletonComponent::class)
abstract class ServiceModule {
    @Binds
    @Singleton
    abstract fun bindActionExecutor(impl: ActionExecutorImpl): ActionExecutor

    @Binds
    @Singleton
    abstract fun bindAccessibilityServiceProvider(impl: AccessibilityServiceProviderImpl): AccessibilityServiceProvider

    @Binds
    @Singleton
    abstract fun bindScreenCaptureProvider(impl: ScreenCaptureProviderImpl): ScreenCaptureProvider

    @Binds
    @Singleton
    abstract fun bindShizukuProvider(impl: ShizukuProviderImpl): ShizukuProvider

    @Binds
    @Singleton
    abstract fun bindAppController(impl: AppControllerImpl): AppController
}
