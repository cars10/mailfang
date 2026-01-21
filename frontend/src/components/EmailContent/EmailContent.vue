<template>
  <div class="flex flex-col grow">
    <div
      v-if="viewMode === 'rendered' && email.body_html"
      class="flex flex-col grow max-w-full"
    >
      <div
        class="flex flex-row items-center justify-between gap-4 p-4 shadow-md"
      >
        <div class="flex flex-row gap-1">
          <ButtonGroup
            v-model="mailLayoutStore.screenSize"
            :options="screenSizeOptions"
          />
          <a
            :href="`/emails/${email.id}/fullscreen`"
            class="btn btn--small"
            target="_blank"
          >
            <ArrowTopRightOnSquareIcon class="h-4 w-4" />
          </a>
        </div>

        <div class="flex flex-row gap-4">
          <Toggle
            v-model="mailLayoutStore.allowRemoteContent"
            label="Remote content"
            small
          />
          <ZoomControls v-model="mailLayoutStore.mailContentZoom" />
        </div>
      </div>

      <div
        v-if="loadingRendered"
        class="flex items-center justify-center h-full"
      >
        <Spinner size="6" />
      </div>
      <div
        v-else
        :class="renderedWrapperClass"
        class="border-t border-gray-200"
      >
        <div :class="iframeWrapperClass" :style="iframeWrapperStyle">
          <iframe
            :key="renderedUrl"
            :src="renderedUrl"
            :class="iframeClass"
            :style="iframeStyle"
          ></iframe>
        </div>
      </div>
    </div>

    <div
      v-if="viewMode === 'text' && email.body_text"
      class="grow whitespace-pre-wrap p-4"
    >
      {{ email.body_text }}
    </div>

    <div v-if="viewMode === 'html' && email.body_html" class="p-4 grow">
      <CodeViewer :content="email.body_html" language="xml" />
    </div>

    <div v-if="viewMode === 'raw'" class="p-4 grow">
      <div v-if="loadingRaw" class="flex items-center justify-center h-full">
        <Spinner size="6" />
      </div>
      <CodeViewer v-else :content="rawContent" />
    </div>

    <EmailSmtpHeaders v-if="viewMode === 'headers'" :email="email" />

    <div v-if="viewMode === 'envelope'" class="p-6">
      <div class="grid grid-cols-[max-content_1fr] gap-x-4 gap-y-3">
        <div class="text-gray-500 select-text text-right text-nowrap py-1">
          MAIL FROM
        </div>
        <div class="text-gray-600 select-text w-fit">
          <CopyBadge :text="email.envelope_from || '(empty)'" />
        </div>

        <div class="text-gray-500 select-text text-right text-nowrap py-1">
          RCPT TO
        </div>
        <div class="w-fit">
          <div
            v-for="recipient in email.recipients"
            :key="recipient"
            class="text-gray-600 select-text w-fit"
          >
            <CopyBadge :text="recipient" />
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
  import { ref, watch, computed } from 'vue'
  import {
    ArrowTopRightOnSquareIcon,
    DevicePhoneMobileIcon,
    DeviceTabletIcon,
    ComputerDesktopIcon,
  } from '@heroicons/vue/24/outline'
  import type { EmailRecord } from '@/types/email'
  import CodeViewer from '@/components/shared/CodeViewer/CodeViewer.vue'
  import { apiClient } from '@/api/client'
  import type { ViewMode } from './EmailView.vue'
  import { useMailLayoutStore, ScreenSize } from '@/stores/MailLayout'
  import ZoomControls from './ZoomControls.vue'
  import EmailSmtpHeaders from './EmailSmtpHeaders.vue'
  import Toggle from '@/components/shared/Toggle/Toggle.vue'
  import ButtonGroup from '@/components/shared/ButtonGroup/ButtonGroup.vue'
  import Spinner from '@/components/shared/Spinner/Spinner.vue'
  import CopyBadge from '@/components/shared/CopyBadge/CopyBadge.vue'

  const props = defineProps<{
    email: EmailRecord
    viewMode: ViewMode
  }>()

  const mailLayoutStore = useMailLayoutStore()
  const rawContent = ref<string>('')
  const loadingRaw = ref(false)
  const loadingRendered = ref(false)

  const screenSizeOptions = [
    {
      value: ScreenSize.Mobile,
      icon: DevicePhoneMobileIcon,
      title: 'Mobile (375px)',
    },
    {
      value: ScreenSize.Tablet,
      icon: DeviceTabletIcon,
      title: 'Tablet (768px)',
    },
    {
      value: ScreenSize.Desktop,
      icon: ComputerDesktopIcon,
      title: 'Desktop',
    },
  ]

  const renderedWrapperClass = computed(() => {
    return [
      'w-full overflow-auto flex justify-center',
      { 'h-full': mailLayoutStore.screenSize === ScreenSize.Desktop },
    ]
  })

  const iframeWrapperClass = computed(() => {
    switch (mailLayoutStore.screenSize) {
      case ScreenSize.Mobile:
        return 'h-full w-full bg-gray-900 border-4 border-gray-800 p-2 rounded-[2.5rem] '
      case ScreenSize.Tablet:
        return 'h-full w-full bg-gray-900 border-4 border-gray-800 p-3 rounded-xl '
      case ScreenSize.Desktop:
        return 'h-full w-full'
      default:
        return ''
    }
  })

  const iframeWrapperStyle = computed(() => {
    switch (mailLayoutStore.screenSize) {
      case ScreenSize.Mobile:
        return {
          maxWidth: '375px',
          width: '375px',
          aspectRatio: '9/18',
          height: 'auto',
          flexShrink: '0',
        }
      case ScreenSize.Tablet:
        return {
          maxWidth: '768px',
          width: '768px',
          aspectRatio: '4/3',
          height: 'auto',
          flexShrink: '0',
        }
      case ScreenSize.Desktop:
        return {
          maxWidth: '100%',
        }
      default:
        return {}
    }
  })

  const iframeClass = computed(() => {
    switch (mailLayoutStore.screenSize) {
      case ScreenSize.Mobile:
        return 'border-0 bg-white rounded-4xl w-full h-full'
      case ScreenSize.Tablet:
        return 'border-0 bg-white rounded-lg h-full w-full'
      case ScreenSize.Desktop:
        return 'border-0 bg-white h-full w-full'
      default:
        return ''
    }
  })

  const iframeStyle = computed(() => ({
    zoom: mailLayoutStore.mailContentZoom,
  }))

  const renderedUrl = computed(() => {
    if (!props.email.body_html) return ''
    const allowRemote = mailLayoutStore.allowRemoteContent
    return `/api/emails/${props.email.id}/rendered?allow_remote_content=${allowRemote}`
  })

  const loadRawEmail = async () => {
    loadingRaw.value = true
    try {
      rawContent.value = await apiClient.getRawEmail(props.email.id)
    } catch (err) {
      console.error('Failed to load raw email:', err)
      rawContent.value = 'Failed to load raw email content'
    } finally {
      loadingRaw.value = false
    }
  }

  watch(
    () => props.viewMode,
    (newValue: ViewMode) => {
      if (newValue === 'raw') loadRawEmail()
    }
  )
</script>
