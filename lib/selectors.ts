import {createSelector} from 'reselect';
import {HyperState} from './muportal';

const getTermGroups = ({termGroups}: Pick<HyperState, 'termGroups'>) => termGroups.termGroups;
export const getRootGroups = createSelector(getTermGroups, (termGroups) =>
  Object.keys(termGroups)
    .map((uid) => termGroups[uid])
    .filter(({parentUid}) => !parentUid)
);
