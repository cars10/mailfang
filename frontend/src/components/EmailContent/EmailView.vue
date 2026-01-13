<template>
  <div class="flex flex-col h-full">
    <div v-if="loading" class="flex items-center justify-center h-full">
      <Spinner size="6" />
    </div>

    <div v-else-if="error" class="flex items-center justify-center h-full">
      <div class="text-gray-500">{{ error }}</div>
    </div>

    <div
      v-else-if="email"
      class="flex flex-col h-full overflow-y-auto bg-gray-50"
    >
      <EmailHeader :email="email" />

      <Tabs v-model="viewMode" :tabs="tabs" />

      <EmailContent :email="email" :view-mode="viewMode" />

      <EmailFooter :email="email" />
    </div>
  </div>
</template>

<script setup lang="ts">
  import { ref, computed, onMounted, watch } from 'vue'
  import { useRoute } from 'vue-router'
  import type { Tab } from '@/components/shared/Tabs/Tabs.vue'
  import { CodeBracketIcon } from '@heroicons/vue/24/outline'
  import { apiClient } from '@/api/client'
  import type { EmailRecord } from '@/types/email'
  import Tabs from '@/components/shared/Tabs/Tabs.vue'
  import EmailHeader from './EmailHeader.vue'
  import EmailContent from './EmailContent.vue'
  import Spinner from '@/components/shared/Spinner/Spinner.vue'
  import EmailFooter from './EmailFooter.vue'

  const route = useRoute()
  const mailId = ref<string>(route.params.id as string)
  const email = ref<EmailRecord | null>(null)
  const loading = ref(true)
  const error = ref<string | null>(null)
  const viewMode = ref<ViewMode>('rendered')

  export type ViewMode =
    | 'rendered'
    | 'html'
    | 'text'
    | 'raw'
    | 'headers'
    | 'envelope'

  const tabs = computed(() => {
    const tabsList: Tab[] = []

    // Left tabs
    if (email.value?.body_html) {
      tabsList.push({ id: 'rendered', label: 'Rendered' })
    }

    if (email.value?.body_text) {
      tabsList.push({ id: 'text', label: 'Text' })
    }

    tabsList.push({ id: 'headers', label: 'Headers', icon: CodeBracketIcon })

    if (email.value?.body_html) {
      tabsList.push({ id: 'html', label: 'HTML', icon: CodeBracketIcon })
    }

    tabsList.push({ id: 'raw', label: 'Raw', icon: CodeBracketIcon })

    tabsList.push({ id: 'envelope', label: 'Envelope' })

    return tabsList
  })

  const fetchEmail = async () => {
    if (!mailId.value) {
      error.value = 'No email ID provided'
      loading.value = false
      return
    }

    try {
      loading.value = true
      error.value = null
      email.value = await apiClient.getEmail(mailId.value)
    } catch (err: unknown) {
      error.value = err instanceof Error ? err.message : 'Failed to fetch email'
    } finally {
      loading.value = false
    }
  }

  watch(
    () => email.value,
    (newEmail: EmailRecord | null) => {
      if (newEmail) {
        viewMode.value = (tabs.value[0]?.id as ViewMode) || 'raw'
      }
    },
    { immediate: true }
  )

  onMounted(fetchEmail)

  watch(
    () => route.params.id,
    newId => {
      mailId.value = newId as string
      fetchEmail()
    }
  )
</script>
