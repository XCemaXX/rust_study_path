use super::*;

unsafe impl<T: Send> Send for LinkedList<T> {}
unsafe impl<T: Sync> Sync for LinkedList<T> {}

unsafe impl<T: Send> Send for Iter<'_, T> {}
unsafe impl<T: Sync> Sync for Iter<'_, T> {}

unsafe impl<T: Send> Send for IterMut<'_, T> {}
unsafe impl<T: Sync> Sync for IterMut<'_, T> {}

unsafe impl<T: Send> Send for CursorMut<'_, T> {}
unsafe impl<T: Sync> Sync for CursorMut<'_, T> {}