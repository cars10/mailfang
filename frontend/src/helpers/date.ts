export const todayTimeOrDate = (dateString: string) => {
  const today = new Date()
  const mailDate = new Date(dateString)

  const isToday =
    mailDate.getDate() === today.getDate() &&
    mailDate.getMonth() === today.getMonth() &&
    mailDate.getFullYear() === today.getFullYear()

  return isToday ? mailDate.toLocaleTimeString() : mailDate.toLocaleDateString()
}

export const formatDate = (dateString: string) => {
  return new Date(dateString).toLocaleString()
}
