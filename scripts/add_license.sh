#!/usr/bin/env sh

PAT_GPL="^// Copyright.*If not, see <http://www.gnu.org/licenses/>\.$"
PAT_OTHER="^// Copyright"

for f in $(find . \( -path ./target -o -path ./utils/ssz \) -prune -o -type f | egrep '\.rs$'); do
	HEADER=$(head -16 $f)
	if [[ $HEADER =~ $PAT_GPL ]]; then
		BODY=$(tail -n +17 $f)
		cat scripts/license_headers/gpl3.rs > temp
		echo "$BODY" >> temp
		mv temp $f
	elif [[ $HEADER =~ $PAT_OTHER ]]; then
		echo "$f: other license was found do nothing"
	else
		echo "$f: added missing header"
		cat scripts/license_headers/gpl3.rs $f > temp
		mv temp $f
	fi
done
