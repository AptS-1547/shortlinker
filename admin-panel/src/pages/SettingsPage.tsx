import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import {
  FiInfo as Info,
  FiServer as Server,
  FiSliders as Sliders,
} from 'react-icons/fi'
import PageHeader from '@/components/layout/PageHeader'
import {
  AboutTab,
  PreferencesTab,
  SystemConfigTab,
} from '@/components/settings'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'

export default function SettingsPage() {
  const { t } = useTranslation()
  const [activeTab, setActiveTab] = useState('preferences')

  return (
    <div className="space-y-6">
      <PageHeader
        title={t('settings.title')}
        description={t('settings.description')}
      />

      <Tabs
        value={activeTab}
        onValueChange={setActiveTab}
        className="space-y-6"
      >
        <TabsList className="grid w-full grid-cols-3 lg:w-auto lg:inline-grid">
          <TabsTrigger value="preferences" className="gap-2">
            <Sliders className="h-4 w-4" />
            <span className="hidden sm:inline">
              {t('settings.tabs.preferences', 'Preferences')}
            </span>
          </TabsTrigger>
          <TabsTrigger value="system" className="gap-2">
            <Server className="h-4 w-4" />
            <span className="hidden sm:inline">
              {t('settings.tabs.system', 'System')}
            </span>
          </TabsTrigger>
          <TabsTrigger value="about" className="gap-2">
            <Info className="h-4 w-4" />
            <span className="hidden sm:inline">
              {t('settings.tabs.about', 'About')}
            </span>
          </TabsTrigger>
        </TabsList>

        <TabsContent value="preferences">
          <PreferencesTab />
        </TabsContent>

        <TabsContent value="system">
          <SystemConfigTab isActive={activeTab === 'system'} />
        </TabsContent>

        <TabsContent value="about">
          <AboutTab />
        </TabsContent>
      </Tabs>
    </div>
  )
}
