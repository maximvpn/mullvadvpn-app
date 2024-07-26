package net.mullvad.mullvadvpn.compose.screen

import android.content.Context
import androidx.compose.foundation.ExperimentalFoundationApi
import androidx.compose.foundation.background
import androidx.compose.foundation.gestures.animateScrollBy
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyItemScope
import androidx.compose.foundation.lazy.LazyListScope
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarDuration
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.rotate
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.compose.dropUnlessResumed
import com.ramcosta.composedestinations.annotation.Destination
import com.ramcosta.composedestinations.annotation.RootGraph
import com.ramcosta.composedestinations.generated.destinations.CreateCustomListDestination
import com.ramcosta.composedestinations.generated.destinations.CustomListEntrySheetDestination
import com.ramcosta.composedestinations.generated.destinations.CustomListLocationsDestination
import com.ramcosta.composedestinations.generated.destinations.CustomListSheetDestination
import com.ramcosta.composedestinations.generated.destinations.CustomListsSheetDestination
import com.ramcosta.composedestinations.generated.destinations.DeleteCustomListDestination
import com.ramcosta.composedestinations.generated.destinations.EditCustomListNameDestination
import com.ramcosta.composedestinations.generated.destinations.FilterDestination
import com.ramcosta.composedestinations.generated.destinations.LocationSheetDestination
import com.ramcosta.composedestinations.navigation.DestinationsNavigator
import com.ramcosta.composedestinations.result.NavResult
import com.ramcosta.composedestinations.result.ResultBackNavigator
import com.ramcosta.composedestinations.result.ResultRecipient
import com.ramcosta.composedestinations.spec.DestinationSpec
import kotlinx.coroutines.launch
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.compose.cell.FilterRow
import net.mullvad.mullvadvpn.compose.cell.HeaderCell
import net.mullvad.mullvadvpn.compose.cell.StatusRelayItemCell
import net.mullvad.mullvadvpn.compose.cell.SwitchComposeSubtitleCell
import net.mullvad.mullvadvpn.compose.cell.ThreeDotCell
import net.mullvad.mullvadvpn.compose.communication.Created
import net.mullvad.mullvadvpn.compose.communication.CustomListAction
import net.mullvad.mullvadvpn.compose.communication.CustomListActionResult
import net.mullvad.mullvadvpn.compose.communication.CustomListSuccess
import net.mullvad.mullvadvpn.compose.communication.Deleted
import net.mullvad.mullvadvpn.compose.communication.GenericError
import net.mullvad.mullvadvpn.compose.communication.LocationsChanged
import net.mullvad.mullvadvpn.compose.communication.Renamed
import net.mullvad.mullvadvpn.compose.component.LocationsEmptyText
import net.mullvad.mullvadvpn.compose.component.MullvadCircularProgressIndicatorLarge
import net.mullvad.mullvadvpn.compose.component.MullvadSnackbar
import net.mullvad.mullvadvpn.compose.component.drawVerticalScrollbar
import net.mullvad.mullvadvpn.compose.constant.ContentType
import net.mullvad.mullvadvpn.compose.extensions.dropUnlessResumed
import net.mullvad.mullvadvpn.compose.state.RelayListItem
import net.mullvad.mullvadvpn.compose.state.SelectLocationUiState
import net.mullvad.mullvadvpn.compose.test.CIRCULAR_PROGRESS_INDICATOR
import net.mullvad.mullvadvpn.compose.test.SELECT_LOCATION_CUSTOM_LIST_HEADER_TEST_TAG
import net.mullvad.mullvadvpn.compose.textfield.SearchTextField
import net.mullvad.mullvadvpn.compose.transitions.SelectLocationTransition
import net.mullvad.mullvadvpn.compose.util.CollectSideEffectWithLifecycle
import net.mullvad.mullvadvpn.compose.util.RunOnKeyChange
import net.mullvad.mullvadvpn.compose.util.showSnackbarImmediately
import net.mullvad.mullvadvpn.lib.model.CustomListId
import net.mullvad.mullvadvpn.lib.model.CustomListName
import net.mullvad.mullvadvpn.lib.model.GeoLocationId
import net.mullvad.mullvadvpn.lib.model.RelayItem
import net.mullvad.mullvadvpn.lib.model.RelayItemId
import net.mullvad.mullvadvpn.lib.theme.AppTheme
import net.mullvad.mullvadvpn.lib.theme.Dimens
import net.mullvad.mullvadvpn.lib.theme.color.AlphaScrollbar
import net.mullvad.mullvadvpn.viewmodel.SelectLocationSideEffect
import net.mullvad.mullvadvpn.viewmodel.SelectLocationViewModel
import org.koin.androidx.compose.koinViewModel

@Preview
@Composable
private fun PreviewSelectLocationScreen() {
    val state =
        SelectLocationUiState.Content(
            searchTerm = "",
            emptyList(),
            relayListItems = emptyList(),
        )
    AppTheme {
        SelectLocationScreen(
            state = state,
        )
    }
}

@Destination<RootGraph>(style = SelectLocationTransition::class)
@Suppress("LongMethod")
@Composable
fun SelectLocation(
    navigator: DestinationsNavigator,
    backNavigator: ResultBackNavigator<Boolean>,
    createCustomListDialogResultRecipient: ResultRecipient<CreateCustomListDestination, Created>,
    editCustomListNameDialogResultRecipient:
        ResultRecipient<EditCustomListNameDestination, Renamed>,
    deleteCustomListDialogResultRecipient: ResultRecipient<DeleteCustomListDestination, Deleted>,
    updateCustomListResultRecipient:
        ResultRecipient<CustomListLocationsDestination, LocationsChanged>,
    locationSheetResultRecipient: ResultRecipient<LocationSheetDestination, CustomListActionResult>,
    customListEntryResultRecipient:
        ResultRecipient<CustomListEntrySheetDestination, CustomListActionResult>
) {
    val vm = koinViewModel<SelectLocationViewModel>()
    val state = vm.uiState.collectAsStateWithLifecycle()

    val snackbarHostState = remember { SnackbarHostState() }
    val context = LocalContext.current
    val lazyListState = rememberLazyListState()
    CollectSideEffectWithLifecycle(vm.uiSideEffect) {
        when (it) {
            SelectLocationSideEffect.CloseScreen -> {
                backNavigator.navigateBack(result = true)
            }
            is SelectLocationSideEffect.LocationAddedToCustomList ->
                launch {
                    snackbarHostState.showResultSnackbar(
                        context = context,
                        result = it.result,
                        onUndo = vm::performAction
                    )
                }
            is SelectLocationSideEffect.LocationRemovedFromCustomList ->
                launch {
                    snackbarHostState.showResultSnackbar(
                        context = context,
                        result = it.result,
                        onUndo = vm::performAction
                    )
                }
            SelectLocationSideEffect.GenericError ->
                launch {
                    snackbarHostState.showSnackbarImmediately(
                        message = context.getString(R.string.error_occurred),
                        duration = SnackbarDuration.Short
                    )
                }
        }
    }

    val stateActual = state.value
    RunOnKeyChange(stateActual is SelectLocationUiState.Content) {
        val index = stateActual.indexOfSelectedRelayItem()
        if (index != -1) {
            lazyListState.scrollToItem(index)
            lazyListState.animateScrollAndCentralizeItem(index)
        }
    }

    createCustomListDialogResultRecipient.OnCustomListNavResult(
        snackbarHostState,
        vm::performAction
    )

    editCustomListNameDialogResultRecipient.OnCustomListNavResult(
        snackbarHostState,
        vm::performAction
    )

    deleteCustomListDialogResultRecipient.OnCustomListNavResult(
        snackbarHostState,
        vm::performAction
    )

    locationSheetResultRecipient.OnCustomListNavResult(snackbarHostState, vm::performAction)

    customListEntryResultRecipient.OnCustomListNavResult(snackbarHostState, vm::performAction)

    updateCustomListResultRecipient.OnCustomListNavResult(snackbarHostState, vm::performAction)

    SelectLocationScreen(
        state = state.value,
        lazyListState = lazyListState,
        snackbarHostState = snackbarHostState,
        onSelectRelay = vm::selectRelay,
        onSearchTermInput = vm::onSearchTermInput,
        onBackClick = dropUnlessResumed { backNavigator.navigateBack() },
        onFilterClick = dropUnlessResumed { navigator.navigate(FilterDestination) },
        removeOwnershipFilter = vm::removeOwnerFilter,
        removeProviderFilter = vm::removeProviderFilter,
        onToggleExpand = vm::onToggleExpand,
        showCustomListBottomSheet =
            dropUnlessResumed { navigator.navigate(CustomListsSheetDestination(true)) },
        showLocationBottomSheet =
            dropUnlessResumed { name, location ->
                navigator.navigate(LocationSheetDestination(name, location))
            },
        showEditCustomListBottomSheet =
            dropUnlessResumed { customListId: CustomListId, customListName: CustomListName ->
                navigator.navigate(CustomListSheetDestination(customListId, customListName))
            },
        showEditCustomListEntryBottomSheet =
            dropUnlessResumed {
                locationName: String,
                customList: CustomListId,
                location: GeoLocationId ->
                navigator.navigate(
                    CustomListEntrySheetDestination(locationName, customList, location)
                )
            },
    )
}

@OptIn(ExperimentalFoundationApi::class)
@Suppress("LongMethod")
@Composable
fun SelectLocationScreen(
    state: SelectLocationUiState,
    lazyListState: LazyListState = rememberLazyListState(),
    snackbarHostState: SnackbarHostState = remember { SnackbarHostState() },
    onSelectRelay: (item: RelayItem) -> Unit = {},
    onSearchTermInput: (searchTerm: String) -> Unit = {},
    onBackClick: () -> Unit = {},
    onFilterClick: () -> Unit = {},
    removeOwnershipFilter: () -> Unit = {},
    removeProviderFilter: () -> Unit = {},
    showCustomListBottomSheet: () -> Unit = {},
    showEditCustomListBottomSheet: (CustomListId, CustomListName) -> Unit = { _, _ -> },
    showEditCustomListEntryBottomSheet: (String, CustomListId, GeoLocationId) -> Unit = { _, _, _ ->
    },
    showLocationBottomSheet: (String, GeoLocationId) -> Unit = { _, _ -> },
    onToggleExpand: (RelayItemId, CustomListId?, Boolean) -> Unit = { _, _, _ -> },
) {
    val backgroundColor = MaterialTheme.colorScheme.background

    Scaffold(
        snackbarHost = {
            SnackbarHost(
                snackbarHostState,
                snackbar = { snackbarData -> MullvadSnackbar(snackbarData = snackbarData) }
            )
        }
    ) {
        Column(modifier = Modifier.padding(it).background(backgroundColor).fillMaxSize()) {
            SelectLocationTopBar(onBackClick = onBackClick, onFilterClick = onFilterClick)

            if (state is SelectLocationUiState.Content && state.filterChips.isNotEmpty()) {
                FilterRow(filters = state.filterChips, removeOwnershipFilter, removeProviderFilter)
            }

            SearchTextField(
                modifier =
                    Modifier.fillMaxWidth()
                        .height(Dimens.searchFieldHeight)
                        .padding(horizontal = Dimens.searchFieldHorizontalPadding),
                backgroundColor = MaterialTheme.colorScheme.tertiaryContainer,
                textColor = MaterialTheme.colorScheme.onTertiaryContainer,
            ) { searchString ->
                onSearchTermInput.invoke(searchString)
            }
            Spacer(modifier = Modifier.height(height = Dimens.verticalSpace))

            LazyColumn(
                modifier =
                    Modifier.fillMaxSize()
                        .drawVerticalScrollbar(
                            lazyListState,
                            MaterialTheme.colorScheme.onBackground.copy(alpha = AlphaScrollbar),
                        ),
                state = lazyListState,
                horizontalAlignment = Alignment.CenterHorizontally,
            ) {
                when (state) {
                    SelectLocationUiState.Loading -> {
                        loading()
                    }
                    is SelectLocationUiState.Content -> {

                        itemsIndexed(
                            items = state.relayListItems,
                            key = { index: Int, item: RelayListItem -> item.key },
                            contentType = { _, item -> item.contentType },
                            itemContent = { index: Int, listItem: RelayListItem ->
                                Column(modifier = Modifier.animateItem()) {
                                    if (index != 0) {
                                        HorizontalDivider(color = backgroundColor)
                                    }
                                    when (listItem) {
                                        RelayListItem.CustomListHeader ->
                                            CustomListHeader(
                                                onShowCustomListBottomSheet = {
                                                    showCustomListBottomSheet()
                                                }
                                            )
                                        is RelayListItem.CustomListItem ->
                                            CustomListItem(
                                                listItem,
                                                onSelectRelay,
                                                {
                                                    showEditCustomListBottomSheet(
                                                        listItem.item.id,
                                                        listItem.item.customList.name
                                                    )
                                                },
                                                { customListId, expand ->
                                                    onToggleExpand(customListId, null, expand)
                                                }
                                            )
                                        is RelayListItem.CustomListEntryItem ->
                                            CustomListEntryItem(
                                                listItem,
                                                { onSelectRelay(listItem.item) },
                                                if (listItem.depth == 1) {
                                                    {
                                                        showEditCustomListEntryBottomSheet(
                                                            listItem.item.name,
                                                            listItem.parentId,
                                                            listItem.item.id
                                                        )
                                                    }
                                                } else {
                                                    null
                                                },
                                                { expand: Boolean ->
                                                    onToggleExpand(
                                                        listItem.item.id,
                                                        listItem.parentId,
                                                        expand
                                                    )
                                                }
                                            )
                                        is RelayListItem.CustomListFooter ->
                                            CustomListFooter(listItem)
                                        RelayListItem.LocationHeader -> RelayLocationHeader()
                                        is RelayListItem.GeoLocationItem ->
                                            RelayLocationItem(
                                                listItem,
                                                { onSelectRelay(listItem.item) },
                                                {
                                                    showLocationBottomSheet(
                                                        listItem.item.name,
                                                        listItem.item.id
                                                    )
                                                },
                                                { expand ->
                                                    onToggleExpand(listItem.item.id, null, expand)
                                                }
                                            )
                                        is RelayListItem.LocationsEmptyText ->
                                            LocationsEmptyText(listItem.searchTerm)
                                    }
                                }
                            }
                        )
                    }
                }
            }
        }
    }
}

@Composable
fun LazyItemScope.RelayLocationHeader() {
    HeaderCell(text = stringResource(R.string.all_locations))
}

@Composable
fun LazyItemScope.RelayLocationItem(
    relayItem: RelayListItem.GeoLocationItem,
    onSelectRelay: () -> Unit,
    onLongClick: () -> Unit,
    onExpand: (Boolean) -> Unit,
) {
    val location = relayItem.item
    StatusRelayItemCell(
        location,
        relayItem.isSelected,
        onClick = { onSelectRelay() },
        onLongClick = { onLongClick() },
        onToggleExpand = { onExpand(it) },
        isExpanded = relayItem.expanded,
        depth = relayItem.depth
    )
}

@Composable
fun LazyItemScope.CustomListItem(
    itemState: RelayListItem.CustomListItem,
    onSelectRelay: (item: RelayItem) -> Unit,
    onShowEditBottomSheet: (RelayItem.CustomList) -> Unit,
    onExpand: ((CustomListId, Boolean) -> Unit),
) {
    val customListItem = itemState.item
    StatusRelayItemCell(
        customListItem,
        itemState.isSelected,
        onClick = { onSelectRelay(customListItem) },
        onLongClick = { onShowEditBottomSheet(customListItem) },
        onToggleExpand = { onExpand(customListItem.id, it) },
        isExpanded = itemState.expanded
    )
}

@Composable
fun LazyItemScope.CustomListEntryItem(
    itemState: RelayListItem.CustomListEntryItem,
    onSelectRelay: () -> Unit,
    onShowEditCustomListEntryBottomSheet: (() -> Unit)?,
    onToggleExpand: (Boolean) -> Unit,
) {
    val customListEntryItem = itemState.item
    StatusRelayItemCell(
        customListEntryItem,
        false,
        onClick = onSelectRelay,
        onLongClick = onShowEditCustomListEntryBottomSheet,
        onToggleExpand = onToggleExpand,
        isExpanded = itemState.expanded,
        depth = itemState.depth
    )
}

@Composable
fun LazyItemScope.CustomListFooter(item: RelayListItem.CustomListFooter) {
    SwitchComposeSubtitleCell(
        text =
            if (item.hasCustomList) {
                stringResource(R.string.to_add_locations_to_a_list)
            } else {
                stringResource(R.string.to_create_a_custom_list)
            },
        modifier = Modifier.background(MaterialTheme.colorScheme.background)
    )
}

@Composable
private fun SelectLocationTopBar(onBackClick: () -> Unit, onFilterClick: () -> Unit) {
    Row(modifier = Modifier.fillMaxWidth()) {
        IconButton(onClick = onBackClick) {
            Icon(
                modifier = Modifier.rotate(270f),
                painter = painterResource(id = R.drawable.icon_back),
                tint = Color.Unspecified,
                contentDescription = null,
            )
        }
        Text(
            text = stringResource(id = R.string.select_location),
            modifier = Modifier.align(Alignment.CenterVertically).weight(weight = 1f),
            textAlign = TextAlign.Center,
            style = MaterialTheme.typography.titleLarge,
            color = MaterialTheme.colorScheme.onPrimary,
        )
        IconButton(onClick = onFilterClick) {
            Icon(
                painter = painterResource(id = R.drawable.icons_more_circle),
                contentDescription = null,
                tint = Color.Unspecified,
            )
        }
    }
}

private fun LazyListScope.loading() {
    item(contentType = ContentType.PROGRESS) {
        MullvadCircularProgressIndicatorLarge(Modifier.testTag(CIRCULAR_PROGRESS_INDICATOR))
    }
}

@Composable
private fun LazyItemScope.CustomListHeader(onShowCustomListBottomSheet: () -> Unit) {
    ThreeDotCell(
        text = stringResource(R.string.custom_lists),
        onClickDots = onShowCustomListBottomSheet,
        modifier = Modifier.testTag(SELECT_LOCATION_CUSTOM_LIST_HEADER_TEST_TAG)
    )
}

private fun SelectLocationUiState.indexOfSelectedRelayItem(): Int =
    if (this is SelectLocationUiState.Content) {
        relayListItems.indexOfFirst {
            when (it) {
                is RelayListItem.CustomListItem -> it.isSelected
                is RelayListItem.GeoLocationItem -> it.isSelected
                is RelayListItem.CustomListEntryItem -> false
                is RelayListItem.CustomListFooter -> false
                RelayListItem.CustomListHeader -> false
                RelayListItem.LocationHeader -> false
                is RelayListItem.LocationsEmptyText -> false
            }
        }
    } else {
        -1
    }

private suspend fun LazyListState.animateScrollAndCentralizeItem(index: Int) {
    val itemInfo = this.layoutInfo.visibleItemsInfo.firstOrNull { it.index == index }
    if (itemInfo != null) {
        val center = layoutInfo.viewportEndOffset / 2
        val childCenter = itemInfo.offset + itemInfo.size / 2
        animateScrollBy((childCenter - center).toFloat())
    } else {
        animateScrollToItem(index)
    }
}

private suspend fun SnackbarHostState.showResultSnackbar(
    context: Context,
    result: CustomListSuccess,
    onUndo: (CustomListAction) -> Unit
) {
    showSnackbarImmediately(
        message = result.message(context),
        actionLabel = context.getString(R.string.undo),
        duration = SnackbarDuration.Long,
        onAction = { onUndo(result.undo) }
    )
}

private fun CustomListSuccess.message(context: Context): String =
    when (this) {
        is Created ->
            locationNames.firstOrNull()?.let { locationName ->
                context.getString(R.string.location_was_added_to_list, locationName, name)
            } ?: context.getString(R.string.locations_were_changed_for, name)
        is Deleted -> context.getString(R.string.delete_custom_list_message, name)
        is Renamed -> context.getString(R.string.name_was_changed_to, name)
        is LocationsChanged -> context.getString(R.string.locations_were_changed_for, name)
    }

@Composable
private fun <D : DestinationSpec, R : CustomListActionResult> ResultRecipient<D, R>
    .OnCustomListNavResult(
    snackbarHostState: SnackbarHostState,
    performAction: (action: CustomListAction) -> Unit
) {
    val scope = rememberCoroutineScope()
    val context = LocalContext.current
    this.onNavResult { result ->
        when (result) {
            NavResult.Canceled -> {
                /* Do nothing */
            }
            is NavResult.Value -> {
                // Handle result
                val customListActionResult = result.value
                when (customListActionResult) {
                    is GenericError -> {
                        scope.launch {
                            snackbarHostState.showSnackbarImmediately(
                                message = context.getString(R.string.error_occurred),
                                duration = SnackbarDuration.Short
                            )
                        }
                    }
                    is CustomListSuccess ->
                        scope.launch {
                            snackbarHostState.showResultSnackbar(
                                context = context,
                                result = customListActionResult,
                                onUndo = performAction
                            )
                        }
                }
            }
        }
    }
}
